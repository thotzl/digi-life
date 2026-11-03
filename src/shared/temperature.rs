//! Water temperature (TCK-202): one model for sensing (`organs::contact::water_temperature`) and thermal stress
//! (`sim::systems::metabolize::EnergyModifier::ThermalStress`). Every world is a water column of the same sea, warm at
//! the surface and cold at the bottom; worlds differ only in whether the thermal stress applies (an energy modifier).

/// Water temperature as a linear function of depth: `surface − (y / height) · span`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemperatureField {
    pub surface: f32,
    pub span: f32,
}

/// Sensor reading at the bottom and its span to the surface (the reading stays within [0.15, 0.95]).
const SENSED_MIN: f32 = 0.15;
const SENSED_SPAN: f32 = 0.8;

impl TemperatureField {
    /// The sea's stratification (38 °C surface, 26 °C span: 12 °C at the bottom).
    pub const WATER: TemperatureField = TemperatureField {
        surface: 38.0,
        span: 26.0,
    };

    /// Temperature in °C at depth `y` of a world of `height`.
    pub fn at(&self, y: f32, height: f32) -> f32 {
        self.surface - (y / height) * self.span
    }

    /// Normalised sensor reading in [0.15, 0.95]: warm surface high, cold bottom low (before the organ's power).
    pub fn sensed(&self, y: f32, height: f32) -> f32 {
        let warmth = ((self.at(y, height) - (self.surface - self.span)) / self.span).clamp(0.0, 1.0);
        SENSED_MIN + SENSED_SPAN * warmth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_and_reading_follow_depth() {
        let w = TemperatureField::WATER;
        assert_eq!((w.at(0.0, 100.0), w.at(100.0, 100.0)), (38.0, 12.0));
        assert!((w.sensed(0.0, 100.0) - 0.95).abs() < 1e-6 && w.sensed(100.0, 100.0) == 0.15);
        assert!(w.sensed(-50.0, 100.0) <= 0.95 + 1e-6 && w.sensed(150.0, 100.0) >= 0.15);
    }
}
