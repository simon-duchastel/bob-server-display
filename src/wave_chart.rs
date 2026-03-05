use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

/// A canvas-based wave chart widget for displaying scrolling wave animations.
pub struct WaveChart<'a> {
    data: &'a [f32],
    color: Color,
    max_points: usize,
    width: Length,
    height: Length,
}

impl<'a> WaveChart<'a> {
    /// Create a new wave chart with the given data and color.
    /// Data values should be in range [0.0, 1.0] where 0.0 is bottom (flat line) and 1.0 is top.
    pub fn new(data: &'a [f32], color: Color) -> Self {
        Self {
            data,
            color,
            max_points: 100,
            width: Length::Fill,
            height: Length::Fixed(40.0),
        }
    }

    /// Set the maximum number of data points to display.
    pub fn max_points(mut self, max_points: usize) -> Self {
        self.max_points = max_points;
        self
    }

    /// Set the width of the chart.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height of the chart.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, Message> From<WaveChart<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(wave_chart: WaveChart<'a>) -> Self {
        Canvas::new(WaveChartProgram {
            data: wave_chart.data,
            color: wave_chart.color,
            max_points: wave_chart.max_points,
        })
        .width(wave_chart.width)
        .height(wave_chart.height)
        .into()
    }
}

struct WaveChartProgram<'a> {
    data: &'a [f32],
    color: Color,
    max_points: usize,
}

impl<'a, Message> canvas::Program<Message> for WaveChartProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut geometry = Vec::new();

        if self.data.is_empty() || bounds.width < 1.0 || bounds.height < 1.0 {
            return geometry;
        }

        // Get the most recent data points (up to max_points)
        let data_to_render: Vec<f32> = self
            .data
            .iter()
            .rev()
            .take(self.max_points)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        if data_to_render.len() < 2 {
            return geometry;
        }

        let point_count = data_to_render.len();
        let x_step = bounds.width / (self.max_points as f32 - 1.0).max(1.0);

        // Create the wave path
        let mut points: Vec<Point> = Vec::with_capacity(point_count);

        for (i, &value) in data_to_render.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            // Clamp value between 0 and 1
            let clamped_value = value.clamp(0.0, 1.0);
            let y = bounds.y + bounds.height - (clamped_value * bounds.height);
            points.push(Point::new(x, y));
        }

        // Create canvas frame and draw
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw filled area under the wave
        let area_path = Path::new(|p| {
            if let Some(first) = points.first() {
                p.move_to(Point::new(first.x, bounds.y + bounds.height));
                for point in &points {
                    p.line_to(*point);
                }
                if let Some(last) = points.last() {
                    p.line_to(Point::new(last.x, bounds.y + bounds.height));
                }
                p.close();
            }
        });

        let fill_color = self.color.scale_alpha(0.3);
        frame.fill(&area_path, fill_color);

        // Draw the main wave line with gradient effect
        // Draw multiple segments with varying alpha for gradient effect
        for i in 0..(points.len().saturating_sub(1)) {
            let recency = i as f32 / points.len() as f32;
            let alpha = 0.5 + recency * 0.5;
            let segment_color = Color {
                a: self.color.a * alpha,
                ..self.color
            };

            let segment_path = Path::line(points[i], points[i + 1]);
            let segment_stroke = Stroke::default().with_color(segment_color).with_width(2.0);

            frame.stroke(&segment_path, segment_stroke);
        }

        geometry.push(frame.into_geometry());
        geometry
    }
}

/// A container for managing historical data for wave charts.
#[derive(Debug, Clone)]
pub struct WaveData {
    values: Vec<f32>,
    capacity: usize,
}

impl WaveData {
    /// Create a new WaveData with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add a new value to the wave data.
    pub fn push(&mut self, value: f32) {
        if self.values.len() >= self.capacity {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    /// Get the current values as a slice.
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

impl Default for WaveData {
    fn default() -> Self {
        Self::new(100)
    }
}
