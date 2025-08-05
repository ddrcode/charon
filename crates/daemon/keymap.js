import { parse } from "yaml";
import { readFileSync } from "fs";

let file = readFileSync('./data/keymaps/pl.yml').toString();
console.log(parse(file));

