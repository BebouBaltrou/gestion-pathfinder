//! Application de bureau (Windows, Linux) : le code est dans `lib.rs`, partagé avec Android.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gestion_pathfinder_lib::run();
}
