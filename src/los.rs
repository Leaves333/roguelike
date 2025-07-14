// let's implement some line of sight algorithms!!!

// draws a line from start to end
// see csustan.csustan.edu/~tom/Lecture-Notes/Graphics/Bresenham-Line.pdf
// en.wikipedia.org/wiki/Bresenham's_line_algorithm#Algorithm
pub fn bresenham(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
    let (x1, y1) = start;
    let (x2, y2) = end;

    let dx = (x2 - y1).abs();
    let dy = (y2 - y1).abs();
    let stepx = { if x2 - x1 > 0 { 1 } else { -1 } };
    let stepy = { if y2 - y1 > 0 { 1 } else { -1 } };

    let mut x = x1;
    let mut y = y1;

    let mut path: Vec<(i32, i32)> = Vec::new();
    path.push((x, y));

    if dx > dy {
        let mut fraction = 2 * dy - dx;
        while x != x2 {
            x += stepx;
            if fraction >= 0 {
                y += stepy;
                fraction -= 2 * dx;
            }
            fraction += 2 * dy;
            path.push((x, y));
        }
    } else {
        let mut fraction = 2 * dx - dy;
        while y != y2 {
            if fraction >= 0 {
                x += stepx;
                fraction -= 2 * dy;
            }
            y += stepy;
            fraction += 2 * dx;
            path.push((x, y));
        }
    }

    return path;
}
