// Sin consola extra en Windows (versión final).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rlp_alumno_lib::run()
}
