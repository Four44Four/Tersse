use tersse::{Color, runtime_terminal_color_code};

#[test]
fn default_color_maps_to_terminal_default_code() {
    assert_eq!(runtime_terminal_color_code(Color::Default), -1);
}

#[test]
fn standard_palette_maps_to_pancurses_constants() {
    assert_eq!(runtime_terminal_color_code(Color::Black), pancurses::COLOR_BLACK);
    assert_eq!(runtime_terminal_color_code(Color::Red), pancurses::COLOR_RED);
    assert_eq!(runtime_terminal_color_code(Color::Green), pancurses::COLOR_GREEN);
    assert_eq!(runtime_terminal_color_code(Color::Yellow), pancurses::COLOR_YELLOW);
    assert_eq!(runtime_terminal_color_code(Color::Blue), pancurses::COLOR_BLUE);
    assert_eq!(
        runtime_terminal_color_code(Color::Magenta),
        pancurses::COLOR_MAGENTA
    );
    assert_eq!(runtime_terminal_color_code(Color::Cyan), pancurses::COLOR_CYAN);
    assert_eq!(runtime_terminal_color_code(Color::White), pancurses::COLOR_WHITE);
}
