module fillet(r) {
    difference() {
        square(r);
        translate([r, r]) circle(r, $fn=50);
    }
}

module fillet_corner(r) {
    difference() {
        cube(r);
        translate([r, r, r]) sphere(r, $fn=50);
    }
}

module case_fillets(r) {
    // Edge fillets
    translate([0, case_true_height, 0])
    rotate([90, 0, 0])
    linear_extrude(case_true_height)
    fillet(r);
    
    translate([case_width, case_true_height, 0])
    rotate([90, 270, 0])
    linear_extrude(case_true_height)
    fillet(r);
   
    translate([case_width, 0, 0])
    rotate([0, -90, 0])
    linear_extrude(case_width)
    fillet(r);
    
    translate([case_width, case_true_height, 0])
    rotate([90, 0, 0])
    rotate([0, -90, 0])
    linear_extrude(case_width)
    fillet(r);
    
    // Fillet corners
    fillet_corner(r);
    
    translate([case_width, 0]) rotate([0, 0, 90])
    fillet_corner(r);
    
    translate([case_width, case_true_height]) rotate([0, 0, 180])
    fillet_corner(r);
    
    translate([0, case_true_height]) rotate([0, 0, 270])
    fillet_corner(r);
}

// --------------

calc_width = 100.75 + 0.2;
calc_height = 159.12 + 0.75;

calc_mounting_hole_offset = 4;
calc_mounting_hole_diameter = 2.6; // about right for m3
calc_mounting_hole_depth = 6;

calc_usb_port_from_bottom = 137; // distance to CENTRE
calc_usb_port_width = 15; // be generous for tolerances and/or big cables

calc_mounting_hole_locations = [
    [0, 0],
    [calc_width - calc_mounting_hole_offset * 2, 0],
    [0, calc_height - calc_mounting_hole_offset * 2],
    [calc_width - calc_mounting_hole_offset * 2, calc_height - calc_mounting_hole_offset * 2],
];

calc_pcb_thickness = 1.65;
calc_underneath_thickness = 2.75;

module calc_cutout() {
    translate([0, 0, calc_mounting_hole_depth - calc_underneath_thickness]) {
        difference() {
            linear_extrude(calc_pcb_thickness + calc_underneath_thickness)
            square([calc_width, calc_height]);

            for (loc = calc_mounting_hole_locations) {
                linear_extrude(calc_underneath_thickness)
                translate(loc)
                square([calc_mounting_hole_offset * 2, calc_mounting_hole_offset * 2]);
            }
        }

        for (loc = calc_mounting_hole_locations) {
            translate(concat(loc + [calc_mounting_hole_offset, calc_mounting_hole_offset], -calc_mounting_hole_depth + calc_underneath_thickness))
            linear_extrude(calc_mounting_hole_depth)
            circle(d = calc_mounting_hole_diameter, $fn = 20);
        }
    }
}

calc_cutout_total_thickness = calc_pcb_thickness + calc_mounting_hole_depth;

// ------------


display_mounting_hole_sep_y = 54.5;
display_mounting_hole_sep_x = 93;
display_mounting_hole_diameter = calc_mounting_hole_diameter;

display_post_width = 8;

display_board_width = 98;
display_board_height = 60;

display_tilt_angle = 15;
display_min_height = 7;

display_pin_cutout_width = 45;
display_pin_cutout_height = 5.5;

display_centre_cutout_width = display_board_width - 14;
display_centre_cutout_height = 48;

module display_frame() {
    difference() {
        rotate([display_tilt_angle, 0])
        translate([0, 0, -20]) // arbitrary
        linear_extrude(20 + display_min_height) // arbitrary
        difference() {
            square([display_board_width, display_board_height]);
            
            // Mounting holes
            translate([display_board_width / 2, display_board_height / 2]) {
                for (mul = [[1, 1], [-1, 1], [1, -1], [-1, -1]]) {
                    translate([display_mounting_hole_sep_x * mul[0] / 2, display_mounting_hole_sep_y * mul[1] / 2])
                    circle(d = case_mounting_hole_diameter, $fn = 20);
                }
                
                translate([-display_centre_cutout_width/2, -display_centre_cutout_height/2])
                square([display_centre_cutout_width, display_centre_cutout_height]);
            }
            
            // Pins
            translate([7, display_board_height - display_pin_cutout_height - 0.5])
            square([display_pin_cutout_width, display_pin_cutout_height]);
        }
        
        // All arbitrary, to cut off the bottom
        translate([0, 0, -100])
        linear_extrude(100)
        square([100, 100]);
    }
}

// ------------

case_height = 200;
case_mounting_hole_offset = 5.5;
case_mounting_hole_diameter = 2.6; // about right for m3

bottom_case_mounting_hole_guide_depth = 6;
bottom_case_mounting_hole_guide_diameter = 6;
bottom_case_calc_border = 8;
bottom_case_depth = 2; // not excluding the height of the PCB
bottom_case_true_depth = calc_cutout_total_thickness + bottom_case_depth;

case_width = calc_width + bottom_case_calc_border * 2;
case_true_height = case_height + bottom_case_calc_border * 2;

bottom_case_logo_indent = 2.5;
bottom_case_fillet_radius = 10;

case_mounting_hole_locations = [
    [0, 0],
    [case_width - case_mounting_hole_offset * 2, 0],
    [0, case_true_height - case_mounting_hole_offset * 2],
    [case_width - case_mounting_hole_offset * 2, case_true_height - case_mounting_hole_offset * 2],
];

module logo() {
    translate([-60 / 2, 3 - 50 / 2])
    minkowski() {
        polygon([
            [0, 0],
            [30, 50],
            [60, 0],
            [0, 0.1],
            [60, 0.1],
            [30, 50.1],
            [0, 0.1]
        ]);
        circle(4, $fn=30);
    }
}

module bottom_case() {
    difference() {
        linear_extrude(calc_cutout_total_thickness + bottom_case_depth)
        minkowski() {
            translate([bottom_case_calc_border, bottom_case_calc_border])
            square([calc_width, case_height]);
            
            circle(r = bottom_case_calc_border, $fn = 20);
        }
        
        translate([bottom_case_calc_border, bottom_case_calc_border, bottom_case_depth])
        calc_cutout();
        
        for (loc = case_mounting_hole_locations) {
            translate(loc + [case_mounting_hole_offset, case_mounting_hole_offset])
            linear_extrude(1000) // arbitrary
            circle(d = case_mounting_hole_diameter, $fn = 20);
            
            translate(loc + [case_mounting_hole_offset, case_mounting_hole_offset])
            linear_extrude(bottom_case_mounting_hole_guide_depth)
            circle(d = bottom_case_mounting_hole_guide_diameter, $fn = 20);
        }
        
        translate([bottom_case_calc_border + calc_width, bottom_case_calc_border + calc_usb_port_from_bottom - calc_usb_port_width / 2])
        linear_extrude(100) // arbitrary
        square([bottom_case_calc_border, calc_usb_port_width]);
        
        translate([case_width / 2, case_true_height / 2])
        linear_extrude(bottom_case_logo_indent)
        logo();
        
        case_fillets(bottom_case_fillet_radius);
    }
    
}

// All from the edge of the PCB, not the case
top_case_button_padding_horizontal = 2;
top_case_button_padding_bottom = 7.6;
top_case_button_padding_top = 36; // TODO: needs pot cutout

top_case_mounting_hole_depth = 5;
top_case_rim_depth = 18.3; // should be roughly around the top of keys

top_case_rim_overhang_spacing = 10; // amount of depth until overhangs for key padding

top_case_pot_x = 12.32; // from left
top_case_pot_y = 28.85; // from top
top_case_pot_diameter = 8;

top_case_reset_button_distance_x = 18;
top_case_reset_button_width = 4.3;
top_case_reset_button_height = 3.5;

module top_case() {
    difference() {
        linear_extrude(top_case_rim_depth)
        difference() {
            minkowski() {
                translate([bottom_case_calc_border, bottom_case_calc_border])
                square([calc_width, case_height]);
                
                circle(r = bottom_case_calc_border, $fn = 20);
            }
            
            // Match cutout with calc PCB on bottom case, except have bonus empty area 
            // for display cables
            translate([bottom_case_calc_border, bottom_case_calc_border])
            square([calc_width, case_true_height - bottom_case_calc_border * 2]);
        }
        
        for (loc = case_mounting_hole_locations) {
            translate(loc + [case_mounting_hole_offset, case_mounting_hole_offset])
            linear_extrude(top_case_mounting_hole_depth)
            circle(d = case_mounting_hole_diameter, $fn = 20);
        }
        
        translate([bottom_case_calc_border + calc_width, bottom_case_calc_border + calc_usb_port_from_bottom - calc_usb_port_width / 2])
        linear_extrude(8)
        square([bottom_case_calc_border, calc_usb_port_width]);
        
        translate([0, case_true_height, top_case_rim_depth])
        rotate([180, 0, 0])
        case_fillets(bottom_case_calc_border);
    }
    
    // Rim padding around buttons
    difference() {
        translate([0, 0, top_case_rim_overhang_spacing])
        linear_extrude(top_case_rim_depth - top_case_rim_overhang_spacing)
        translate([bottom_case_calc_border, bottom_case_calc_border]) {
            square([top_case_button_padding_horizontal, calc_height]);
            
            translate([calc_width - top_case_button_padding_horizontal, 0])
            square([top_case_button_padding_horizontal, calc_height]);
            
            square([calc_width, top_case_button_padding_bottom]);
            
            // This one needs a cutout for the contrast pot and reset button
            difference() {
                translate([0, calc_height - top_case_button_padding_top])
                square([calc_width, top_case_button_padding_top]);
                
                translate([top_case_pot_x, calc_height - top_case_pot_y])
                circle(d = top_case_pot_diameter, $fn = 20);
            }
            
            // Not actually around buttons - fills in area above display cables
            difference() {
                translate([0, calc_height])
                square([calc_width, case_height - calc_height]);
                
                // Continue cable cutout from display_frame
                // Unfortunately a bit of guesswork with these multipliers
                translate([(case_width - display_board_width - 1.5) / 2, case_true_height - bottom_case_calc_border * 2.6])
                square([display_pin_cutout_width, display_pin_cutout_height]);
                
                translate([(case_width - display_board_width - 1.5) / 2, case_true_height - bottom_case_calc_border * 8])
                square([display_centre_cutout_width, display_centre_cutout_height]);
            }
        }
        
        // Cut out reset button hole
        translate([case_width - top_case_reset_button_distance_x - top_case_reset_button_width, calc_usb_port_from_bottom + 10])
        rotate([15, 0, 0])
        linear_extrude(1000)
        square([top_case_reset_button_width, top_case_reset_button_height]);
    }
    
    // Display
    translate([(case_width - display_board_width) / 2, case_true_height - bottom_case_calc_border - display_board_height - 1, top_case_rim_depth])
    display_frame();
}

module bootsel_button() {
    difference() {
        rotate([-15, 0, 0])
        linear_extrude(30)
        square([top_case_reset_button_width - 0.15, top_case_reset_button_height - 0.15]);
        
        translate([0, 0, -10])
        linear_extrude(10)
        square([50, 50]);
        
        translate([0, 0, 19])
        linear_extrude(100)
        square([50, 50]);
    }
}

//bottom_case();
//translate([0, 0, bottom_case_true_depth]) top_case();
bootsel_button();

