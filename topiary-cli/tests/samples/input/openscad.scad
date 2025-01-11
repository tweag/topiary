include <my_path/my_lib.scad>
use <my_path/my_lib.scad>
// variables
rr = a_vector[2]; // member of vector
range1 = [-1.5:0.5:3]; // for() loop range
xx = [0:5]; // alternate for() loop range
$fn = 360; // special variable
E = 2.71828182845904523536028747135266249775724709369995; // constant
cond_var = "is_true" ? true : false;

function line(point1, point2, width = 1) =

  let (angle = 90 - atan((point2[1] - point1[1]) / (point2[0] - point1[0])))

  let (offset_x = 0.5 * width * cos(angle), offset_y = 0.5 * width * sin(angle))

  let (offset1 = [ -offset_x, offset_y], offset2 = [offset_x, -offset_y])

  // [P1a, P2a, P2b, P1b]
  [point1 + offset1, point2 + offset1, point2 + offset2, point1 + offset2];

// ================================================================================
// Transformations
// ================================================================================
cylinder();
cylinder(
    d = 5,
    h = 100,
);
rotate([90, 0, 0]) cylinder();
translate([1, 0, 0]) {
    difference() {
        rotate([0, 90, 0]) cylinder();
        cube();
    }
}

// ================================================================================
// Nested Items
// ================================================================================
module big_module(){ function inner_function() = undef;
module inner_module()cube();
}

// module extern_module() include <other_file.scad>
//
// for (i = [10:50])
// {
//     let (angle = i*360/20, r= i*2, distance = r*5)
//     {
//         rotate(angle, [1, 0, 0])
//         translate([0, distance, 0])
//         sphere(r = r);
//     }
// }
//
// if ($preview)
//     if (false)
//         sphere();
//     else
//         translate([2, 0, 0]) cube();
//
// for (i = [1:2:7]) {
//     let (x = i ^ 2, y = x - 1) {
//         translate([x, y, 0]) sphere(r = i);
//     }
// }
//
// intersection_for (i = [1, 2, 3]) {
//     if (i > 1) {
//         translate([0, i, 0]) cube();
//     }
// }
//
// // ================================================================================
// // Comments
// // ================================================================================
// /* ignored [Customizer Group] ignored */
// /* Multiline
// comment
// here
// */
// my_parameter = 5;
//
// function math(x) = /*do math stuff*/ x + 2 // done with math
// ;
//
// module my_cylinder() {
//   // here we create a cylinder
//   cylinder(); /* done ! */
//   cube();
// }
