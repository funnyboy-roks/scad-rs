use std::{
    borrow::Cow,
    io::{self, Write},
};

use bauer::Builder;

use crate::{
    ToScad,
    boolean::{Difference, Intersection, Union},
    dim::_2D,
    math::{ScadValue, Vector2},
    modifiers::{Disabled, Highlight, ShowOnly, Transparent},
    shape::ClosureShape,
    shape3d::LinearExtrude,
    transform::{Rotated, Scaled, Translated},
};

pub trait Shape2d: ToScad + Sized {
    fn disable(self) -> Disabled<_2D, Self> {
        Disabled::new(self)
    }

    fn show_only(self) -> ShowOnly<_2D, Self> {
        ShowOnly::new(self)
    }

    fn highlight(self) -> Highlight<_2D, Self> {
        Highlight::new(self)
    }

    fn transparent(self) -> Transparent<_2D, Self> {
        Transparent::new(self)
    }

    fn translate(self, translation: impl Into<Vector2>) -> Translated<_2D, Self> {
        Translated::new(self, translation.into())
    }

    fn rotate(self, rotation: impl Into<ScadValue>) -> Rotated<_2D, Self> {
        Rotated::new(self, rotation.into())
    }

    fn scale(self, scale: impl Into<f64>) -> Scaled<_2D, Self> {
        Scaled::new(self, scale.into())
    }

    fn difference<R>(self, other: R) -> Difference<_2D, Self, R> {
        Difference::new(self, other)
    }

    fn union<R>(self, other: R) -> Union<_2D, Self, R> {
        Union::new(self, other)
    }

    fn intersection<R>(self, other: R) -> Intersection<_2D, Self, R> {
        Intersection::new(self, other)
    }

    fn linear_extrude(self, height: impl Into<ScadValue>) -> LinearExtrude<Self> {
        LinearExtrude::new(self, height.into())
    }

    fn number_of_segments(self, n: u32) -> impl Shape2d {
        ClosureShape::new(move |writer: &mut dyn Write| {
            write!(writer, "let ($fn = {}) {{", n)?;
            self.to_scad(writer)?;
            write!(writer, "}}")
        })
    }
}

impl<T> Shape2d for &T where T: Shape2d {}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_shape_2d_inner {
    // impl['a, T] for Foo<'a, T>
    (impl$([$($tt: tt)*])? for $ty: ty) => {
        impl<$($($tt)*,)? Rhs: $crate::shape2d::Shape2d> std::ops::Sub<Rhs> for $ty {
            type Output = $crate::boolean::Difference<$crate::dim::_2D, Self, Rhs>;

            fn sub(self, rhs: Rhs) -> Self::Output {
                $crate::shape2d::Shape2d::difference(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape2d::Shape2d> std::ops::Add<Rhs> for $ty {
            type Output = $crate::boolean::Union<$crate::dim::_2D, Self, Rhs>;

            fn add(self, rhs: Rhs) -> Self::Output {
                $crate::shape2d::Shape2d::union(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape2d::Shape2d> std::ops::BitOr<Rhs> for $ty {
            type Output = $crate::boolean::Union<$crate::dim::_2D, Self, Rhs>;

            fn bitor(self, rhs: Rhs) -> Self::Output {
                $crate::shape2d::Shape2d::union(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape2d::Shape2d> std::ops::BitAnd<Rhs> for $ty {
            type Output = $crate::boolean::Intersection<$crate::dim::_2D, Self, Rhs>;

            fn bitand(self, rhs: Rhs) -> Self::Output {
                $crate::shape2d::Shape2d::intersection(self, rhs)
            }
        }
    };
}

/// Implement [`Shape2d`] and some binary operations on a struct
#[macro_export]
macro_rules! impl_shape_2d {
    // impl['a, T] for Foo<'a, T>
    (impl$([$($tt: tt)*])? for $ty: ty) => {
        $crate::impl_shape_2d_inner!(impl$([$($tt)*])? for $ty);
        impl$(<$($tt)*>)? $crate::shape2d::Shape2d for $ty {}
        $crate::impl_shape_2d_inner!(impl$([$($tt)*])? for &$ty);
    };
}

pub struct Circle {
    radius: ScadValue,
}
impl_shape_2d!(impl for Circle);

impl Circle {
    pub fn with_radius(radius: impl Into<ScadValue>) -> Self {
        Self {
            radius: radius.into(),
        }
    }

    pub fn with_diameter(diameter: impl Into<ScadValue>) -> Self {
        Self {
            radius: diameter.into() / 2.,
        }
    }
}

impl ToScad for Circle {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "circle(r = ")?;
        self.radius.to_scad(writer)?;
        write!(writer, ");")
    }
}

#[derive(Debug, Default)]
pub enum TextDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl ToScad for TextDirection {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        let s = match self {
            TextDirection::LeftToRight => "ltr",
            TextDirection::RightToLeft => "rtl",
            TextDirection::TopToBottom => "ttb",
            TextDirection::BottomToTop => "btt",
        };
        s.to_scad(writer)
    }
}

#[derive(Debug, Default)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl ToScad for HorizontalAlign {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        let s = match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        };
        s.to_scad(writer)
    }
}

#[derive(Debug, Default)]
pub enum VerticalAlign {
    Top,
    Center,
    #[default]
    Baseline,
    Bottom,
}

impl ToScad for VerticalAlign {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        let s = match self {
            Self::Top => "top",
            Self::Center => "center",
            Self::Baseline => "baseline",
            Self::Bottom => "bottom",
        };
        s.to_scad(writer)
    }
}

/// text(t, size, font, direction, language, script, halign, valign, spacing)
///
/// See <https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Text>
#[derive(Debug, Builder)]
pub struct Text<'a> {
    #[builder(into)]
    text: Cow<'a, str>,
    /// The generated text has an ascent (height above the baseline) of approximately this value.
    /// Default is 10. Fonts vary and may be a different height, typically slightly smaller. The
    /// formula to convert the size value to "points" is `pt = size * 3.937`, so a size argument of
    /// 3.05 will give about 12pt text, for instance.
    ///
    /// Note: if you know a point is 1/72" this may not look right, but point measurements of text
    /// are the distance from ascent to descent, not from ascent to baseline as in this case.
    #[builder(default = "10.", into)]
    size: ScadValue,
    /// The name of the font that should be used. This is not the name of the font file, but the
    /// logical font name (internally handled by the fontconfig library). This can also include a
    /// style parameter, see below. A list of installed fonts & styles can be obtained using the
    /// font list dialog (Help -> Font List).
    #[builder(into)]
    font: Option<Cow<'a, str>>,
    /// Direction of the text flow
    direction: Option<TextDirection>,
    /// The language of the text (e.g., "en", "ar", "ch"). Default is "en"
    #[builder(into)]
    language: Option<Cow<'a, str>>,
    /// The script of the text (e.g., "latin", "arabic", "hani"). Default is "latin"
    #[builder(into)]
    script: Option<Cow<'a, str>>,
    /// The horizontal alignment for the text
    halign: Option<HorizontalAlign>,
    /// The vertical alignment for the text
    valign: Option<VerticalAlign>,
    /// Factor to increase/decrease the character spacing. The default value of 1 results in the
    /// normal spacing for the font, giving a value greater than 1 causes the letters to be spaced
    /// further apart.
    #[builder(into)]
    spacing: Option<ScadValue>,
}

impl ToScad for Text<'_> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "text(")?;
        self.text.to_scad(writer)?;
        write!(writer, ", size = ")?;
        self.size.to_scad(writer)?;

        if let Some(font) = &self.font {
            write!(writer, ", font = ")?;
            font.to_scad(writer)?;
        }
        if let Some(direction) = &self.direction {
            write!(writer, ", direction = ")?;
            direction.to_scad(writer)?;
        }
        if let Some(language) = &self.language {
            write!(writer, ", language = ")?;
            language.to_scad(writer)?;
        }
        if let Some(script) = &self.script {
            write!(writer, ", script = ")?;
            script.to_scad(writer)?;
        }
        if let Some(halign) = &self.halign {
            write!(writer, ", halign = ")?;
            halign.to_scad(writer)?;
        }
        if let Some(valign) = &self.valign {
            write!(writer, ", valign = ")?;
            valign.to_scad(writer)?;
        }
        if let Some(spacing) = &self.spacing {
            write!(writer, ", spacing = ")?;
            spacing.to_scad(writer)?;
        }
        write!(writer, ");")?;
        Ok(())
    }
}

impl_shape_2d!(impl['a] for Text<'a>);
