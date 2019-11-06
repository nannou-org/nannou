# Drawing 2D Shapes

In this tutorial we explore drawing 2D shapes with Nannou.  We will cover drawing basic lines, simple polygons (e.g. ellipses, rectangles, etc.), and more complex polygons (where you can create whatever shape you'd like)!

To begin with, we will need a Nannou project file to work with.  Copy the following into new file: 

```rust,no_run
use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to purple.
    draw.background().color(PLUM);

    // Draw a blue ellipse with default size and position.
    draw.ellipse().color(STEELBLUE);

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}
```

You can also find this file, and other useful examples, in the [examples](https://github.com/nannou-org/nannou/tree/master/examples) directory of the Nannou source repository.

## Drawing Simple Shapes

Let's try running the file!  (if you haven't already, you will need to add this file to your Cargo.toml file)

You should a new window with something that looks like this:

![A simple circle](./images/2d-shape-circle.png)

Already we are rendering a circle to our canvas.  As you may have guessed, the line of code responsible for creating a circle is the call to the `ellipse` function:

```
draw.ellipse().color(STEELBLUE);
```

There are many ways we can alter our circle here.  Let's start with changing the size:

```
draw.ellipse().color(STEELBLUE)
              .w(300.0)
              .h(200.0);
```

The `w` function here changes the width of the ellipse to 300 pixels, and the `h` function changes the height to 200.0 pixels. You should see what we would more colloquially refer to as an ellipse.  

We can also change the position of our ellipse with the `x_y` method:

```
draw.ellipse().color(STEELBLUE)
              .w(300.0)
              .h(200.0)
              .x_y(200.0, -100.0);
```

![An ellipse](./images/2d-shape-ellipse.png)

As you can see, we edit our ellipse by chaining together different methods which will change one or more properties of our shape.  This is called the **Builder** pattern.  The call to `draw.ellipse()` returns an object of type `Drawing<Ellipse>`.  In turn, each call to a builder method, such as `w(300.0)` or `x_y(200.0, -100.0)`, returns the same instance of our shape. By chaining these function calls, we are able to build an ellipse with the attributes we want.

There are several more methods we can use to build our ellipse. You can view the documentation for many of these methods [here](https://docs.rs/nannou/latest/nannou/draw/struct.Drawing.html).

### Drawing Rectangles and Quadrilaterals
Drawing a square or rectangle uses the same builder pattern that drawing an ellipse does.  In fact, it's similar enough that you can swap out `ellipse` with `rect` in the example above to get a working example:

```
draw.rect().color(STEELBLUE)
           .w(300.0)
           .h(200.0);
```

You will see an image like this:

![A rectangle](./images/2d-shape-rect.png)

In addition to `rect`, you can also use the `quad` method, which is for drawing quadrilaterals. This function is similar to `rect`, but you can also choose to supply your own coordinates for your shape.  Try the following:

```
let point1 = pt2(-10.0, -20.0);
let point2 = pt2(10.0, -30.0);
let point3 = pt2(15.0, 40.0);
let point4 = pt2(-20.0, 35.0);

draw.quad()
    .color(STEELBLUE)
    .w(300.0)
    .h(200.0)
    .points(point1, point2, point3, point4);
```

You should see the following:

![A quadrilateral with custom defined points](./images/2d-shape-quad.png)

The `pt2` method above will create a point object that represents a point in XY coordinate space, like a graph or a Cartesian plane. Nannou's coordinate system places (0,0) at the center of the window. This is **not** like many other graphical creative coding frameworks, which place (0,0) at the upper-leftmost position of the window. 

Note that while the `Drawing` builder objects for different shapes share many of the same builder methods, they do not share all of them.  Trying to use the method `points` on an instance of an `Drawing<Ellipse>`, for example, will raise an error.  

### Drawing a Triangle

Additionally, there is one more simple shape method: `tri`, for drawing triangles.  It behaves similarly to `quad`, where you can supply your own coordinates to decide how the shape looks.  Try it out!

![A triangle](./images/2d-shape-tri.png)

## Drawing Lines

The `line` function provides a simple way to draw a line:

```
let start_point = pt2(-30.0, -20.0);
let end_point   = pt2(40.0, 40.0);

draw.line()
    .start(start_point)
    .end(end_point)
    .weight(4.0)
    .color(STEELBLUE);
```

![A simple line](./images/2d-simple-line.png)

Simply provide a starting point and an ending point, and you have your line.

This is great for simpler drawings, but what if you want to draw something more complicated? A sine wave, for instance.

To draw our sine wave, we will use the `polyline` function.  To use this function, we will supply a collection (or array) of points that represent points on a sine wave.  We can generate this array of points using&mdash;what else&mdash;the `sin` function!

```
let points = (0..50).map(|i| {
      let x = (i as f32 - 25.0);          //subtract 25 to center the sine wave
      let point = pt2(x, x.sin()) * 20.0; //scale sine wave by 20.0
      (point, STEELBLUE)
    });
draw.polyline()
    .weight(3.0)
    .colored_points(points);
```

![A sine wave polyline drawing](./images/2d-simple-polyline.png)

As you can see, the power of `polyline` is the ability to draw a series of lines
connecting and ordered array of points.  With this, you can easily draw a
variety of shapes or lines, so long as you can provide or generate the points
you need to represent that shape.

For example, a circle:

```
let radius = 150.0;                   // store the radius of the circle we want to make
let points = (0..=360).map(|i| {      // map over an array of integers from 0 to 360 to represent the degrees in a circle

   let radian = deg_to_rad(i as f32); // convert each degree to radians
   let x = radian.sin() * radius;     // get the sine of the radian to find the x-co-ordinate of
                                      // this point of the circle, and multiply it by the radius
   let y = radian.cos() * radius;     // do the same with cosine to find the y co-ordinate
   (pt2(x,y), STEELBLUE)              // construct and return a point object with a color
});
draw.polyline()                       // create a PathStroke Builder object
    .weight(3.0)
    .colored_points(points);          // tell our PathStroke Builder to draw lines connecting our array of points
```
![A custom circle](./images/2d-custom-circle-outline.png)

A custom drawn circle! ...okay, perhaps this isn't too exciting, given that we
already have an easy way of drawing circles with `ellipse`.  But with a simple change to the above code we can generate an outline of a
different shape.  Let's try using the `step_by` function, which allows us to
choose the interval at which we would like to step through a range or other iterator.  So instead
of calling `(0..=360).map`, we will call `(0..=360).step_by(45).map`:

```
let points = (0..=360).step_by(45).map(|i| {
```

The rest of our code will remain unchanged.

Because 45 divides into 360 eight times, our code generated 8 points to represent a regular octagon.

![An octagon outline](./images/2d-complete-octogon-outline.png)

An octagon!

Try experimenting with different values to pass into `step_by` and see the
different shapes you can create!

As a side note, you may have noticed that we did not use a `color` function to set the drawing's
color this time.  Instead, `polyline` requires that each point be given a color.
This means that you can change the color of the polyline point-by-point.  Try
experimenting with it!

## Drawing Custom Polygons

To draw a custom filled-in polygon (and not just an outline), will we use code very similar to our custom circle or
octagon code.  The main difference is that instead of calling `polyline` to
create a Builder, we call `polygon`:

```
let radius = 150.0;
let points = (0..=360).step_by(45).map(|i| {
   let radian = deg_to_rad(i as f32);
   let x = radian.sin() * radius;
   let y = radian.cos() * radius;
   pt2(x,y)
});
draw.polygon()
    .color(STEELBLUE)
    .points(points);
```

![An octagon](./images/2d-custom-octogon-polygon.png)

Notice how we are again using the `color` function to set the color of our
polygon, similar to the basic polygon functions covered in the beginning of this
tutorial.

## Concluding Remarks

In this tutorial, we learned about most basic 2D drawing functions with Nannou.

You can view the documentation for the different `Drawing` objects these return here:

* [Ellipse](https://docs.rs/nannou/latest/nannou/draw/primitive/ellipse/struct.Ellipse.html)
* [Rect](https://docs.rs/nannou/latest/nannou/draw/primitive/rect/struct.Rect.html)
* [Quad](https://docs.rs/nannou/latest/nannou/draw/primitive/quad/struct.Quad.html)
* [Tri](https://docs.rs/nannou/latest/nannou/draw/primitive/tri/struct.Tri.html)
* [Polyline (or PathStroke)](https://docs.rs/nannou/latest/nannou/draw/primitive/path/type.PathStroke.html)
* [Polygon](https://docs.rs/nannou/latest/nannou/draw/primitive/polygon/struct.Polygon.html)

These links provide more information about other functions you can use to change your drawings in a variety of ways.

You have now learned about some of the most commonly used functions for 2D drawing with
Nannou. Of course, this is just scratching the surface of ways in which you can generate
shapes or polygons with Nannou, but it should serve as a solid starting point in
creating your own drawings.

Happy coding!
