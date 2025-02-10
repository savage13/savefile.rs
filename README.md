# BOTW SaveData Viewer / Editor

## Install

```
git clone https://github.com/savage13/savefile.rs.git
cd savefile.rs
cargo install --path .
botw-editor
```

## Examples

```botw-editor -i game_data.sav -v 'Clear_Dungeon*'
Clear_Dungeon012 true
Clear_Dungeon112 true
Clear_Dungeon116 true
...
```
```botw-editor -i game_data.sav -v '*Npc*Korok*'
MainField_Npc_HiddenKorokGround_3148789636 true
MainField_Npc_HiddenKorokGround_779408426 true
MainField_Npc_HiddenKorokGround_158421429 true
MainField_Npc_HiddenKorokFly_2151114466 true
...
```


## License

BSD 2-Clause
