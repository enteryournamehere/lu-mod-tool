use crate::locale::Localization;
use crate::locale::Phrase;
use crate::locale::Translation;
use crate::mod_context::ModContext;
use assembly_fdb::{core::Field, mem::Database, store};
use color_eyre::eyre::{self, eyre, WrapErr};
use mapr::Mmap;
use std::collections::HashMap;
use std::path::Path;
use std::{fmt::Write, fs::File, io::BufWriter, io::Write as _, path::PathBuf, time::Instant};
use structopt::StructOpt;
mod locale;
mod lu_mod;
mod manifest;
mod mod_context;
mod mods;
use crate::lu_mod::*;
use crate::manifest::Manifest;
use crate::mods::Mods;
mod component;
use crate::component::{component_name_to_id, component_name_to_table_name};
use rusqlite::{params_from_iter, Connection};

#[derive(StructOpt)]
#[structopt(author = "zaop")]
#[structopt(
    name = "lu-mod-tool",
    about = "Rust port of Wincent's InfectedRose.Interface.\n\
    This is currently incomplete:\n\
     - Option --copy is not supported\n\
     - Currently only the action \"add\" is supported, not edit or remove\n\
     - No fancy coloured terminal output :("
)]
struct Options {
    #[structopt(
        short = "i",
        long = "input",
        help = "Path to mods.json.",
        default_value = "mods.json"
    )]
    input: PathBuf,

    #[allow(dead_code)]
    #[structopt(
        short = "c",
        long = "copy",
        help = "[unimplemented] Generate mods to copy this object LOT."
    )]
    copy_object: Option<u32>,

    #[allow(dead_code)]
    #[structopt(
        short = "d",
        long = "id",
        help = "[unimplemented] The id of the mod objects generated by copy.",
        default_value = "my-object"
    )]
    copy_id: String,

    #[allow(dead_code)]
    #[structopt(
        short = "o",
        long = "output",
        help = "[unimplemented] The file to output the generated mods to (when generating mods).",
        default_value = "output.json"
    )]
    output: PathBuf,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opts = Options::from_args();

    let start_time = Instant::now();

    println!("Input: {:?}", opts.input);

    let configuration = read_or_create_json::<Mods>(&PathBuf::from(&opts.input))?;

    std::env::set_current_dir(opts.input.parent().unwrap())?;

    // Step one. Open database.
    print!("Opening database... ");
    std::io::stdout().flush()?;
    let timer = Instant::now();
    // CDCLIENT.FDB
    let database_source_path = &configuration.database;
    let database_destination_path = "../res/cdclient.fdb";
    if !database_source_path.is_file() {
        std::fs::copy(database_destination_path, database_source_path)?;
    }

    let src_file = File::open(&database_source_path).wrap_err_with(|| {
        format!(
            "Failed to open input file '{}'",
            database_source_path.display()
        )
    })?;
    let mmap = unsafe { Mmap::map(&src_file)? };
    let buffer: &[u8] = &mmap;

    let timer = print_timer(timer);

    // LOCALE.XML
    let locale_source_path = Path::new("locale.xml");
    let locale_destination_path = Path::new("../locale/locale.xml");
    if !locale_source_path.is_file() {
        std::fs::copy(locale_destination_path, locale_source_path)?;
    }
    // read source xml into mod_context.Localization
    print!("Reading locale... ");
    std::io::stdout().flush()?;
    let localization = read_xml::<Localization>(locale_source_path)?;

    let timer = print_timer(timer);

    let mut mod_context = ModContext::<'_> {
        root: std::env::current_dir()?,
        configuration,
        database: Database::new(buffer),
        localization,
        ids: Default::default(),
        mods: Default::default(),
        server_sql: Default::default(),
    };

    // verify opts.input exists
    if !&opts.input.is_file() {
        return Err(eyre!("Input file does not exist"));
    }

    // TODO check version

    // TODO update mods.json if changed

    // TODO handle Copy Object.

    // TODO priorities.

    println!("Applying mods.");
    // Find all directories with a manifest.json file.
    let mut mods_dirs = Vec::new();
    for entry in std::fs::read_dir(".")? {
        let path = entry?.path();
        if path.is_dir() && path.join("manifest.json").is_file() {
            mods_dirs.push(path);
        }
    }
    // Loop over dirs.
    for mods_dir in mods_dirs {
        // Apply manifest.
        let manifest_path = &mods_dir.join("manifest.json");
        apply_manifest(&mut mod_context, manifest_path)?;
    }

    // println!("{:#?}", mod_context.mods);

    let mut lookup: HashMap<String, i32> = HashMap::new();

    // from here on down a lot should be rewritten to be clearer and probably more efficient

    let mut object_mods = mod_context
        .mods
        .iter_mut()
        .filter(|m| m.mod_type == "object")
        .collect::<Vec<&mut Mod>>();

    let object_mods_count = object_mods.len();

    let mut ids = {
        let objects_table = get_table(&mod_context.database, "Objects")?;
        find_available_ids(&objects_table, object_mods_count)?
    };
    // println!("{} needed, first {:?}", object_mods_count, ids);

    // assign IDs
    for added_object in &mut object_mods {
        let id = ids.pop().unwrap();
        added_object.fields[0] = Field::Integer(id);
        lookup.insert(added_object.id.clone(), id);

        // add locale entries
        if !&mod_context.localization.phrases.phrase.is_empty() {
            let mut phrase = Phrase {
                id: format!("Objects_{}_name", id),
                translations: vec![],
            };
            for (language, text) in &added_object.locale {
                phrase.translations.push(Translation {
                    locale: language.to_string(),
                    value: text.to_string(),
                })
            }
            mod_context.localization.phrases.phrase.push(phrase);
        }
    }

    let mut relevant_component_tables: Vec<String> = vec![];

    for modification in mod_context.mods.iter() {
        match modification.mod_type.as_str() {
            "object" | "sql" => continue, // TODO add npc and stuff ?? unsure if those are used
            _ => {
                if relevant_component_tables.contains(&modification.mod_type) {
                    continue;
                }
                let table_name =
                    component_name_to_table_name(modification.mod_type.as_str())?.clone();
                relevant_component_tables.push(table_name);
            }
        }
    }

    for component_name in relevant_component_tables {
        let component_mods = get_mods_for_table(&mod_context, component_name.as_str());
        // println!("{} count = {}", component_name, component_mods.len());
        let component_table = get_table(&mod_context.database, component_name.as_str())?;
        let ids = find_available_ids(&component_table, component_mods.len())?;
        let mut component_mods = get_mods_for_table_mut(&mut mod_context, component_name.as_str());
        for (i, added_component) in component_mods.iter_mut().enumerate() {
            if !added_component.fields.is_empty() {
                added_component.fields[0] = Field::Integer(ids[i]);
            }
            lookup.insert(added_component.id.clone(), ids[i]);
        }
    }

    let mut component_registry: Vec<Vec<Field>> = vec![];

    // Create component registry
    for modification in mod_context.mods.iter() {
        if modification.mod_type == "object" {
            let linked_components = &modification.components;
            for linked_component_name in linked_components {
                let linked_component = &mod_context
                    .mods
                    .iter()
                    .find(|m| &m.id == linked_component_name)
                    .unwrap(); // bad
                let component_number = component_name_to_id(linked_component.mod_type.as_str())?;
                // println!("{} has component {}", id, linked_component_name);
                let component_id = lookup.get(linked_component_name).unwrap(); //bad
                component_registry.push(vec![
                    Field::Integer((modification.fields[0].clone().into_opt_integer()).unwrap()),
                    Field::Integer(component_number),
                    Field::Integer(*component_id),
                ]);
            }
        }
    }

    print!("Applied mods in ");
    let timer = print_timer(timer);

    // Create destination database and merge new rows into it
    print!("Building output database... ");
    std::io::stdout().flush()?;
    let mut dest_fdb = store::Database::new();
    let sqlite_path = Path::new(&mod_context.configuration.sqlite);
    // delete the sqlite file if it exists
    if sqlite_path.exists() {
        std::fs::remove_file(sqlite_path)?;
    }
    let dest_sqlite = Connection::open(&sqlite_path)?;
    dest_sqlite.execute("BEGIN", rusqlite::params![])?;
    for src_table in mod_context.database.tables()?.iter() {
        let src_table = src_table?;
        let mut create_query = format!("CREATE TABLE IF NOT EXISTS \"{}\"\n(\n", src_table.name());
        let mut insert_query = format!("INSERT INTO \"{}\" (", src_table.name());

        let to_add = {
            match src_table.name().into_owned().as_str() {
                "Objects" => get_rows_for_insertion(&mod_context, "object"),
                "ComponentsRegistry" => component_registry.clone(),
                _ => get_rows_for_insertion(&mod_context, src_table.name().into_owned().as_str()),
            }
        };

        let unique_key_count = to_add.len() + src_table.bucket_count();
        let new_bucket_count = if unique_key_count == 0 {
            0
        } else {
            u32::next_power_of_two(unique_key_count as u32)
        };

        let mut dest_table = store::Table::new(new_bucket_count as usize);

        let mut first = true;
        for src_column in src_table.column_iter() {
            // sqlite
            if first {
                first = false;
            } else {
                writeln!(create_query, ",").unwrap();
                write!(insert_query, ", ").unwrap();
            }
            let column_type = src_column.value_type().to_sqlite_type();
            write!(create_query, "    [{}] {}", src_column.name(), column_type).unwrap();
            write!(insert_query, "[{}]", src_column.name()).unwrap();

            // fdb
            dest_table.push_column(src_column.name_raw(), src_column.value_type());
        }
        // sqlite
        create_query.push_str(");");
        insert_query.push_str(") VALUES (?1");
        for i in 2..=src_table.column_count() {
            write!(insert_query, ", ?{}", i).unwrap();
        }
        insert_query.push_str(");");
        dest_sqlite.execute(&create_query, rusqlite::params![])?;

        let mut insert_statement = dest_sqlite.prepare(&insert_query)?;
        for addable in to_add {
            if addable.is_empty() {
                continue;
            }
            // sqlite
            insert_statement.execute(params_from_iter(addable.iter()))?;
            // fdb
            let pk = match &addable[0] {
                Field::Integer(i) => *i as usize,
                Field::BigInt(i) => *i as usize,
                Field::Text(t) => (sfhash::digest(t.as_bytes())) as usize,
                _ => return Err(eyre!("Cannot use {:?} as primary key", &addable[0])),
            };
            dest_table.push_row(pk, &addable);
        }

        let mut row_buffer: Vec<Field> = Vec::with_capacity(src_table.column_count());

        for (pk, src_bucket) in src_table.bucket_iter().enumerate() {
            for src_row in src_bucket.row_iter() {
                for field in src_row.field_iter() {
                    row_buffer.push(Field::from(field));
                }
                // sqlite
                insert_statement.execute(params_from_iter(row_buffer.iter()))?;
                // fdb
                dest_table.push_row(pk, &row_buffer[..]);
                row_buffer.clear();
            }
        }

        dest_fdb.push_table(src_table.name_raw(), dest_table);
    }

    let timer = print_timer(timer);

    print!("Applying SQL mods... ");
    std::io::stdout().flush()?;
    for modification in mod_context.mods.iter() {
        if modification.mod_type == "sql" {
            let sql = modification.values.get("sql").unwrap();
            // type was checked earlier
            let mut sql_str = String::from(sql.as_str().unwrap());
            // remove transactions because they cannot be nested
            sql_str = sql_str.replace("BEGIN TRANSACTION;", "");
            sql_str = sql_str.replace("COMMIT;", "");
            dest_sqlite.execute(sql_str.as_str(), rusqlite::params![])?;
        }
    }
    let timer = print_timer(timer);

    print!("Exporting SQLite... ");
    std::io::stdout().flush()?;
    dest_sqlite.execute("COMMIT", rusqlite::params![])?;

    let timer = print_timer(timer);

    print!("Exporting FDB... ");
    std::io::stdout().flush()?;
    let fdb_path = Path::new("../res/cdclient.fdb");
    let dest_file = File::create(fdb_path)?;
    let mut dest_out = BufWriter::new(dest_file);
    dest_fdb
        .write(&mut dest_out)
        .wrap_err("Failed to write output database")?;

    let timer = print_timer(timer);

    // serialize &mod_context.localization
    print!("Exporting locale... ");
    std::io::stdout().flush()?;

    mod_context.localization.locales.count = mod_context.localization.locales.locale.len() as i32;
    mod_context.localization.phrases.count = mod_context.localization.phrases.phrase.len() as i32;
    write_xml(&mod_context.localization, Path::new("../locale/locale.xml"))?;
    let _ = print_timer(timer);

    println!("\nGenerated IDs:");
    let mut keys = lookup.keys().collect::<Vec<&String>>();
    keys.sort();
    for key in keys {
        println!(" {:>5} : {}", lookup[key], key);
    }

    let duration = start_time.elapsed();
    println!(
        "\nFinished in {}.{:#03}s",
        duration.as_secs(),
        duration.subsec_millis()
    );

    Ok(())
}

fn apply_manifest(mod_context: &mut ModContext, manifest_path: &Path) -> eyre::Result<()> {
    let manifest = read_json::<Manifest>(manifest_path)?;

    println!("Applying {}", &manifest.name);
    for mod_file in &manifest.files {
        let real_path = &manifest_path.parent().unwrap().join(mod_file);
        println!("  └ {:?}", &real_path);
        apply_mod_file(mod_context, &manifest, real_path)?;
    }
    Ok(())
}

fn apply_mod_file(
    mod_context: &mut ModContext,
    _manifest: &Manifest,
    file: &Path,
) -> eyre::Result<()> {
    let mods: Vec<Mod> = read_json::<Vec<Mod>>(file)?;
    let dir = file.parent().unwrap();
    for mut lu_mod in mods {
        println!("    └ {:?}", &lu_mod.id);
        lu_mod.dir = dir.into();

        // #[allow(unused_must_use)]
        let _ = match lu_mod.mod_type.as_str() {
            "item" => apply_item_mod(&mut lu_mod)?,
            "sql" => apply_sql_mod(mod_context, &mut lu_mod)?,
            "environmental" => apply_environmental_mod(&lu_mod)?,
            "mission" => apply_mission_mod(&lu_mod)?,
            "npc" => apply_npc_mod(&lu_mod)?,
            "object" => apply_object_mod(mod_context, &mut lu_mod)?,
            _ => add_row(mod_context, &mut lu_mod)?,
        };

        println!("      └ {:?} fields", &lu_mod.fields.len());

        mod_context.mods.push(lu_mod.clone()); // ehhh
    }
    Ok(())
}

fn get_mods_for_table_mut<'a>(
    mod_context: &'a mut ModContext,
    target_table: &str,
) -> Vec<&'a mut Mod> {
    mod_context
        .mods
        .iter_mut()
        .filter(
            |modification| match component_name_to_table_name(modification.mod_type.as_str()) {
                Ok(table_name) => table_name == target_table,
                Err(_) => false,
            },
        )
        .collect()
}

fn get_mods_for_table<'a>(mod_context: &'a ModContext, target_table: &str) -> Vec<&'a Mod> {
    mod_context
        .mods
        .iter()
        .filter(
            |modification| match component_name_to_table_name(modification.mod_type.as_str()) {
                Ok(table_name) => table_name == target_table,
                Err(_) => false,
            },
        )
        .collect()
}

fn get_rows_for_insertion(mod_context: &ModContext, target_table: &str) -> Vec<Vec<Field>> {
    get_mods_for_table(mod_context, target_table)
        .iter()
        .map(|modification| modification.fields.clone())
        .collect()
}

fn read_json<T>(path: &Path) -> eyre::Result<T>
where
    T: serde::de::DeserializeOwned + Default + serde::Serialize + std::fmt::Debug,
{
    let contents = std::fs::read_to_string(&path)?;
    let json: T = serde_json::from_str(&contents)?;
    Ok(json)
}

fn read_or_create_json<T>(path: &Path) -> eyre::Result<T>
where
    T: serde::de::DeserializeOwned + Default + serde::Serialize + std::fmt::Debug,
{
    if path.exists() {
        read_json(path)
    } else {
        let json: T = Default::default();
        let mut file = BufWriter::new(File::create(&path)?);
        serde_json::to_writer(&mut file, &json)?;
        Ok(json)
    }
}

fn read_xml<T>(path: &Path) -> eyre::Result<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug,
{
    let contents = std::fs::read_to_string(&path)?;
    let xml: T = quick_xml::de::from_str(&contents)?;
    Ok(xml)
}

fn write_xml<T>(content: T, path: &Path) -> eyre::Result<()>
where
    T: serde::Serialize + std::fmt::Debug,
{
    let mut writer = BufWriter::new(File::create(path)?);
    let _ = writer.write(b"<?xml version=\"1.0\" encoding=\"UTF-8\" ?>");
    quick_xml::se::to_writer(&mut writer, &content)?;
    Ok(())
}

fn print_timer(start: Instant) -> Instant {
    let duration = start.elapsed();
    println!("{}ms", duration.as_millis());
    Instant::now()
}
