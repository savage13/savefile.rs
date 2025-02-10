# BOTW SaveData Viewer / Editor

## Install

```
git clone https://github.com/savage13/savefile.rs.git
cd savefile.rs
cargo install --path .
botw-editor
```

## Examples

```
% botw-editor -h
BotW Save Editor

Usage: botw-editor [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    game_data.sav input file
  -v, --value <VALUE>    name to read, accepts wildcards * and ?
  -s, --set <SET>        set name=value
  -o, --output <OUTPUT>  output file
  -w, --writeover        overwrite the input file
  -a, --all              show all values (name, value, hash(name))
  -h, --help             Print help
  -V, --version          Print version
```

```
% botw-editor -i game_data.sav -v 'Clear_Dungeon*'
Clear_Dungeon012 true
Clear_Dungeon112 true
Clear_Dungeon116 true
...
```

```
% botw-editor -i game_data.sav -v '*Npc*Korok*'
MainField_Npc_HiddenKorokGround_3148789636 true
MainField_Npc_HiddenKorokGround_779408426 true
MainField_Npc_HiddenKorokGround_158421429 true
MainField_Npc_HiddenKorokFly_2151114466 true
...
```



## License

BSD 2-Clause
