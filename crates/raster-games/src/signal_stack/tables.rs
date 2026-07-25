// SPDX-License-Identifier: MPL-2.0

use super::{CellPoint, Packet, Rotation};

pub(super) const PACKET_ORDER: [Packet; 7] = [
    Packet::I,
    Packet::J,
    Packet::L,
    Packet::O,
    Packet::S,
    Packet::T,
    Packet::Z,
];

const I: [[CellPoint; 4]; 4] = [
    points([(3, 2), (4, 2), (5, 2), (6, 2)]),
    points([(5, 1), (5, 2), (5, 3), (5, 4)]),
    points([(3, 3), (4, 3), (5, 3), (6, 3)]),
    points([(4, 1), (4, 2), (4, 3), (4, 4)]),
];
const J: [[CellPoint; 4]; 4] = [
    points([(3, 2), (3, 3), (4, 3), (5, 3)]),
    points([(5, 2), (4, 2), (4, 3), (4, 4)]),
    points([(5, 4), (5, 3), (4, 3), (3, 3)]),
    points([(3, 4), (4, 4), (4, 3), (4, 2)]),
];
const L: [[CellPoint; 4]; 4] = [
    points([(5, 2), (3, 3), (4, 3), (5, 3)]),
    points([(5, 4), (4, 2), (4, 3), (4, 4)]),
    points([(3, 4), (3, 3), (4, 3), (5, 3)]),
    points([(3, 2), (4, 2), (4, 3), (4, 4)]),
];
const O: [[CellPoint; 4]; 4] = [
    points([(4, 2), (5, 2), (4, 3), (5, 3)]),
    points([(4, 2), (5, 2), (4, 3), (5, 3)]),
    points([(4, 2), (5, 2), (4, 3), (5, 3)]),
    points([(4, 2), (5, 2), (4, 3), (5, 3)]),
];
const S: [[CellPoint; 4]; 4] = [
    points([(4, 2), (5, 2), (3, 3), (4, 3)]),
    points([(4, 2), (4, 3), (5, 3), (5, 4)]),
    points([(5, 3), (4, 3), (4, 4), (3, 4)]),
    points([(3, 2), (3, 3), (4, 3), (4, 4)]),
];
const T: [[CellPoint; 4]; 4] = [
    points([(4, 2), (3, 3), (4, 3), (5, 3)]),
    points([(4, 2), (4, 3), (5, 3), (4, 4)]),
    points([(3, 3), (4, 3), (5, 3), (4, 4)]),
    points([(4, 2), (3, 3), (4, 3), (4, 4)]),
];
const Z: [[CellPoint; 4]; 4] = [
    points([(3, 2), (4, 2), (4, 3), (5, 3)]),
    points([(5, 2), (4, 3), (5, 3), (4, 4)]),
    points([(3, 3), (4, 3), (4, 4), (5, 4)]),
    points([(4, 2), (3, 3), (4, 3), (3, 4)]),
];

const fn points(values: [(i8, i8); 4]) -> [CellPoint; 4] {
    [
        CellPoint::new(values[0].0, values[0].1),
        CellPoint::new(values[1].0, values[1].1),
        CellPoint::new(values[2].0, values[2].1),
        CellPoint::new(values[3].0, values[3].1),
    ]
}

pub(super) const fn cells(packet: Packet, rotation: Rotation) -> &'static [CellPoint; 4] {
    let index = rotation as usize;
    match packet {
        Packet::I => &I[index],
        Packet::J => &J[index],
        Packet::L => &L[index],
        Packet::O => &O[index],
        Packet::S => &S[index],
        Packet::T => &T[index],
        Packet::Z => &Z[index],
    }
}

type Kicks = [(i8, i8); 5];

const ZERO_KICKS: Kicks = [(0, 0); 5];

pub(super) const fn kicks(packet: Packet, from: Rotation, to: Rotation) -> &'static Kicks {
    if matches!(packet, Packet::O) {
        return &ZERO_KICKS;
    }
    if matches!(packet, Packet::I) {
        return i_kicks(from, to);
    }
    jlstz_kicks(from, to)
}

const fn jlstz_kicks(from: Rotation, to: Rotation) -> &'static Kicks {
    match (from, to) {
        (Rotation::Zero, Rotation::Right) | (Rotation::Two, Rotation::Right) => {
            &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)]
        }
        (Rotation::Right, Rotation::Zero) | (Rotation::Right, Rotation::Two) => {
            &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)]
        }
        (Rotation::Two, Rotation::Left) | (Rotation::Zero, Rotation::Left) => {
            &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]
        }
        (Rotation::Left, Rotation::Two) | (Rotation::Left, Rotation::Zero) => {
            &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]
        }
        _ => &ZERO_KICKS,
    }
}

const fn i_kicks(from: Rotation, to: Rotation) -> &'static Kicks {
    match (from, to) {
        (Rotation::Zero, Rotation::Right) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (Rotation::Right, Rotation::Zero) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (Rotation::Right, Rotation::Two) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        (Rotation::Two, Rotation::Right) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (Rotation::Two, Rotation::Left) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (Rotation::Left, Rotation::Two) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (Rotation::Left, Rotation::Zero) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (Rotation::Zero, Rotation::Left) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        _ => &ZERO_KICKS,
    }
}
