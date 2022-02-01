Rust port of Wincent's [InfectedRose.Interface](https://github.com/Wincent01/InfectedRose/tree/modding/InfectedRose.Interface#readme).

It is not finished; only **adding** to the database is possible at the moment, editing and removing is not, and neither is exporting from the DB. Not all mod types are supported, see the list below.
Apart from these limitations, it can be used in the same way as InfectedRose.Interface, and the goal is to be fully compatible with the mod format and command line interface.

Supported mod types:
- [x] SQL
- [x] Object + separately defined components
- [x] Environmental
- [x] Item
- [x] NPC
- [ ] Enemy
- [x] Mission
- [ ] Zone
- [ ] Skill
