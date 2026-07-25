// SPDX-License-Identifier: MPL-2.0

/// A cell position in a logical grid.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GridPoint {
    pub x: u16,
    pub y: u16,
}

impl GridPoint {
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// A logical grid size.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GridSize {
    pub width: u16,
    pub height: u16,
}

impl GridSize {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn area(self) -> usize {
        self.width as usize * self.height as usize
    }

    #[must_use]
    pub const fn contains(self, point: GridPoint) -> bool {
        point.x < self.width && point.y < self.height
    }
}

/// A rectangular region in a logical grid.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GridRect {
    pub origin: GridPoint,
    pub size: GridSize,
}

impl GridRect {
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            origin: GridPoint::new(x, y),
            size: GridSize::new(width, height),
        }
    }

    #[must_use]
    pub const fn from_size(size: GridSize) -> Self {
        Self::new(0, 0, size.width, size.height)
    }

    #[must_use]
    pub const fn contains(self, point: GridPoint) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x < self.right()
            && point.y < self.bottom()
    }

    #[must_use]
    pub const fn right(self) -> u16 {
        self.origin.x.saturating_add(self.size.width)
    }

    #[must_use]
    pub const fn bottom(self) -> u16 {
        self.origin.y.saturating_add(self.size.height)
    }

    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        let left = self.origin.x.max(other.origin.x);
        let top = self.origin.y.max(other.origin.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Self::new(
            left,
            top,
            right.saturating_sub(left),
            bottom.saturating_sub(top),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_clips_to_shared_area() {
        let left = GridRect::new(2, 3, 5, 4);
        let right = GridRect::new(5, 1, 4, 5);

        assert_eq!(left.intersection(right), GridRect::new(5, 3, 2, 3));
    }

    #[test]
    fn disjoint_intersection_is_empty() {
        let left = GridRect::new(0, 0, 2, 2);
        let right = GridRect::new(4, 4, 2, 2);

        assert_eq!(left.intersection(right).size.area(), 0);
    }
}
