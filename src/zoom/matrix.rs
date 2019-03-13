use std::fmt::{Display, Formatter, Result};

use web_sys::{SvgMatrix, SvgRect};

#[derive(Debug, Clone)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    #[inline]
    pub fn new(x: f32, y: f32) -> Point2D {
        Point2D { x, y }
    }

    #[inline]
    pub fn matrix_transform(&self, matrix: &Matrix2D) -> Point2D {
        Point2D {
            x: (self.x * matrix.a) + (self.y * matrix.c) + matrix.e,
            y: (self.x * matrix.b) + (self.y * matrix.d) + matrix.f,
        }
    }
}

impl Display for Point2D {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let precision = f.precision().unwrap_or(3);
        let width = f.width().unwrap_or(6);

        write!(
            f,
            "({:w$.p$}, {:w$.p$})",
            self.x,
            self.y,
            w = width,
            p = precision
        )
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub top_left: Point2D,
    pub bottom_right: Point2D,
}

impl Rect {
    #[inline]
    pub fn new(top_left: Point2D, bottom_right: Point2D) -> Rect {
        Rect {
            top_left,
            bottom_right,
        }
    }

    pub fn from_svg(js_rect: &SvgRect) -> Rect {
        Rect::new(
            Point2D {
                x: js_rect.x(),
                y: js_rect.y(),
            },
            Point2D {
                x: js_rect.x() + js_rect.width(),
                y: js_rect.y() + js_rect.height(),
            },
        )
    }

    pub fn matrix_transform(&self, matrix: &Matrix2D) -> Rect {
        Rect {
            top_left: self.top_left.matrix_transform(matrix),
            bottom_right: self.bottom_right.matrix_transform(matrix),
        }
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.top_left.x
    }
    #[inline]
    pub fn top(&self) -> f32 {
        self.top_left.y
    }
    #[inline]
    pub fn right(&self) -> f32 {
        self.bottom_right.x
    }
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.bottom_right.y
    }

    #[inline]
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.top_left.x - self.bottom_right.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.top_left.y - self.bottom_right.y
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.top_left.fmt(f)?;
        write!(f, " -> ")?;
        self.bottom_right.fmt(f)?;

        Ok(())
    }
}

/// [a c e]
/// [b d f]
///
#[derive(Debug, Clone)]
pub struct Matrix2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix2D {
    #[inline]
    pub fn from_js(js_matrix: &SvgMatrix) -> Matrix2D {
        Matrix2D {
            a: js_matrix.a(),
            b: js_matrix.b(),
            c: js_matrix.c(),
            d: js_matrix.d(),
            e: js_matrix.e(),
            f: js_matrix.f(),
        }
    }
}

impl Display for Matrix2D {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let precision = f.precision().unwrap_or(3);
        let width = f.width().unwrap_or(7);

        write!(
            f,
            "[{:w$.p$}, {:w$.p$}, {:w$.p$}]\n[{:w$.p$}, {:w$.p$}, {:w$.p$}]",
            self.a,
            self.c,
            self.e,
            self.b,
            self.d,
            self.f,
            w = width,
            p = precision
        )
    }
}
