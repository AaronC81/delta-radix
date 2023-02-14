keycap_height = 7;
legend_height = 1.5;
keycap_size = 18;
legend_font_size = 14;

module keycap_with_legend(txt, legend_y_offset=0, legend_x_offset=0) {
    difference() {
        import("keycap_base.stl");

        translate([keycap_size / 2 - .25 + legend_x_offset, (keycap_size - legend_font_size) / 2 + legend_y_offset, keycap_height - legend_height])
        legend(txt, legend_height);
    }
}

module legend(txt, height) {
    linear_extrude(height)
    offset(delta = -0.1)
    text(txt, size = legend_font_size, halign = "center", font = "SF Mono:style=Bold");
}

// ------------------

module number_and_letter_keycaps() {
    s = 25;

    translate([s*0, s*0]) keycap_with_legend("O");

    translate([s*0, s*1]) keycap_with_legend("1");
    translate([s*1, s*1]) keycap_with_legend("2");
    translate([s*2, s*1]) keycap_with_legend("3");

    translate([s*0, s*2]) keycap_with_legend("4");
    translate([s*1, s*2]) keycap_with_legend("5");
    translate([s*2, s*2]) keycap_with_legend("6");

    translate([s*0, s*3]) keycap_with_legend("7");
    translate([s*1, s*3]) keycap_with_legend("8");
    translate([s*2, s*3]) keycap_with_legend("9");

    translate([s*1, s*0]) keycap_with_legend("A");
    translate([s*2, s*0]) keycap_with_legend("B");
    translate([s*3, s*0]) keycap_with_legend("C");
    translate([s*3, s*1]) keycap_with_legend("D");
    translate([s*3, s*2]) keycap_with_legend("E");
    translate([s*3, s*3]) keycap_with_legend("F");
}

module number_and_letter_legends() {
    s = 18;
    h = legend_height - 0.2;
    
    translate([s*0, s*0]) legend("O", h);

    translate([s*0, s*1]) legend("1", h);
    translate([s*1, s*1]) legend("2", h);
    translate([s*2, s*1]) legend("3", h);

    translate([s*0, s*2]) legend("4", h);
    translate([s*1, s*2]) legend("5", h);
    translate([s*2, s*2]) legend("6", h);

    translate([s*0, s*3]) legend("7", h);
    translate([s*1, s*3]) legend("8", h);
    translate([s*2, s*3]) legend("9", h);

    translate([s*1, s*0]) legend("A", h);
    translate([s*2, s*0]) legend("B", h);
    translate([s*3, s*0]) legend("C", h);
    translate([s*3, s*1]) legend("D", h);
    translate([s*3, s*2]) legend("E", h);
    translate([s*3, s*3]) legend("F", h);
}

// ------------------

module operator_keycaps() {
    s = 25;

    translate([s*0, s*0]) keycap_with_legend("+", 1);
    translate([s*1, s*0]) keycap_with_legend("−", 1);
    translate([s*2, s*0]) keycap_with_legend("×", 1);
    translate([s*3, s*0]) keycap_with_legend("÷", 1);
    
    translate([s*0, s*1]) keycap_with_legend("x", 1.7);
    translate([s*1, s*1]) keycap_with_legend("b", 0);
    translate([s*2, s*1]) keycap_with_legend("→", 0, 0.3);
}

module operator_legends() {
    s = 18;
    h = legend_height - 0.2;

    translate([s*0, s*0]) legend("+", h);
    translate([s*1, s*0]) legend("−", h);
    translate([s*2, s*0]) legend("×", h);
    translate([s*3, s*0]) legend("÷", h);
    
    translate([s*0, s*1]) legend("x", h);
    translate([s*1, s*1]) legend("b", h);
    translate([s*2, s*1]) legend("→", h);
}

// ------------------

// TODO
module special_keycaps() {
    s = 25;

    translate([s*0, s*0]) keycap_with_legend("^", -3); // Shift
    translate([s*2, s*1]) keycap_with_legend("=", 1);  // Exe
    translate([s*1, s*0]) keycap_with_legend("◦", 1);  // Menu

    translate([s*0, s*1]) keycap_with_legend("<", 1);  // Left
    translate([s*1, s*1]) keycap_with_legend(">", 1);  // Right
    
    translate([s*2, s*0]) keycap_with_legend("⌫", 0);  // Del
    
    translate([s*3, s*0]) keycap_with_legend("?", 0);  // Var
}

module special_legends() {
    s = 18;
    h = legend_height - 0.2;

    //translate([s*0, s*0]) legend("^", h); // Shift
    //translate([s*2, s*1]) legend("=", h);  // Exe
    //translate([s*1, s*0]) legend("◦", h);  // Menu

    translate([s*0, s*1]) legend("<", h);  // Left
    translate([s*1, s*1]) legend(">", h);  // Right
    
    //translate([s*2, s*0]) legend("⌫", h);  // Del
    
    //translate([s*3, s*0]) legend("?", h);  // Var
}

special_legends();

