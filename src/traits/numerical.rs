use bigdecimal::{BigDecimal, Signed, ToPrimitive};
use std::convert::TryFrom;

pub trait Percentage {
    fn percent(&self, percent: Self) -> Self;
    fn percent_fraction(&self, percent: f32) -> Self;
    fn percent_of_other(&self, other: Self) -> Self;
}

pub trait CanRound {
    #[must_use]
    fn round_2dp(&self) -> Self;
}

impl CanRound for f32 {
    fn round_2dp(&self) -> Self {
        BigDecimal::try_from(*self)
            .unwrap()
            .with_scale(2)
            .to_f32()
            .unwrap()
    }
}

impl CanRound for BigDecimal {
    fn round_2dp(&self) -> Self {
        self.with_scale(2)
    }
}

impl Percentage for f32 {
    /// Get a percentage of the provided value.
    ///
    /// # Arguments
    ///
    /// * `percent`: The percentage of the value to return
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v = 50f32.percent(10f32);
    /// ```
    fn percent(&self, percent: Self) -> Self {
        if percent < 0f32 {
            panic!("Percentage can't be negative")
        }
        (self / 100f32) * percent
    }

    /// Get a percentage of the provided value
    ///
    /// # Arguments
    ///
    /// * `fraction`: The percentage to return provided in decimal form
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =50f32.percent_fraction(0.1f32);
    /// ```
    fn percent_fraction(&self, fraction: f32) -> Self {
        if fraction < 0f32 {
            panic!("Fraction can't be negative")
        }
        self * fraction
    }

    /// Express this value as a percentage of another value.
    ///
    /// # Arguments
    ///
    /// * `other`: The value you want to compare the source value to.
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =10f32.percent_of_other(50f32);
    /// ```
    fn percent_of_other(&self, other: Self) -> Self {
        (self / other) * 100f32
    }
}

impl Percentage for u32 {
    /// Get a percentage of the provided value.
    ///
    /// # Arguments
    ///
    /// * `percent`: The percentage of the value to return
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =50f32.percent(10f32); // returns 10% of 50 = 5
    /// ```
    fn percent(&self, percent: Self) -> Self {
        (self / 100u32) * percent
    }

    /// Get a percentage of the provided value
    ///
    /// # Arguments
    ///
    /// * `fraction`: The percentage to return provided in decimal form
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =50f32.percent_fraction(0.1f32);
    /// ```
    fn percent_fraction(&self, fraction: f32) -> Self {
        if fraction < 0f32 {
            panic!("Fraction can't be negative")
        }
        (*self as f32 * fraction) as Self
    }

    /// Express this value as a percentage of another value.
    ///
    /// # Arguments
    ///
    /// * `other`: The value you want to compare the source value to.
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =10f32.percent_of_other(50f32);
    /// ```
    fn percent_of_other(&self, other: Self) -> Self {
        ((self / other) as f32 * 100f32) as Self
    }
}

impl Percentage for i32 {
    /// Get a percentage of the provided value.
    ///
    /// # Arguments
    ///
    /// * `percent`: The percentage of the value to return
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =50f32.percent(10f32);
    /// ```
    fn percent(&self, percent: Self) -> Self {
        if percent < 0i32 {
            panic!("Percentage can't be negative")
        }
        (self / 100i32) * percent
    }

    /// Get a percentage of the provided value
    ///
    /// # Arguments
    ///
    /// * `fraction`: The percentage to return provided in decimal form
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v =50i32.percent_fraction(0.1f32);
    /// ```
    fn percent_fraction(&self, fraction: f32) -> Self {
        if fraction < 0f32 {
            panic!("Fraction can't be negative")
        }
        (*self as f32 * fraction) as Self
    }

    /// Express this value as a percentage of another value.
    ///
    /// # Arguments
    ///
    /// * `other`: The value you want to compare the source value to.
    ///
    /// returns: f32
    ///
    /// # Examples
    ///
    /// ```
    /// use deepfreeze::traits::numerical::Percentage;
    /// let v = 10i32.percent_of_other(50i32);
    /// ```
    fn percent_of_other(&self, other: Self) -> Self {
        ((self / other) as f32 * 100f32) as Self
    }
}

impl Percentage for BigDecimal {
    fn percent(&self, percent: Self) -> Self {
        if !percent.is_positive() {
            panic!("Percentage can't be negative or zero")
        }
        (self / BigDecimal::from(100)) * percent
    }

    fn percent_fraction(&self, percent: f32) -> Self {
        if percent < 0f32 {
            panic!("Percentage can't be negative")
        }
        let bd_pct = BigDecimal::from(percent);
        self * bd_pct
    }

    fn percent_of_other(&self, other: Self) -> Self {
        let bd_pct = BigDecimal::from(100);
        (self / other) * bd_pct
    }
}
