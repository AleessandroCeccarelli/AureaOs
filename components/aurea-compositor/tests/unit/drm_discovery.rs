// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

#[test]
fn derives_stable_ids_from_connector_ids() {
    assert_eq!(output_id_for_connector(1, 10), (1_u128 << 32) | 10);

    assert_eq!(output_id_for_connector(1, 20), (1_u128 << 32) | 20);
}

#[test]
fn connector_ids_do_not_depend_on_enumeration_order() {
    let first = output_id_for_connector(1, 42);
    let second = output_id_for_connector(1, 17);

    assert_eq!(first, output_id_for_connector(1, 42));
    assert_eq!(second, output_id_for_connector(1, 17));
}

#[test]
fn different_devices_produce_different_output_ids() {
    let first_device = output_id_for_connector(1, 10);
    let second_device = output_id_for_connector(2, 10);

    assert_ne!(first_device, second_device);
}

#[test]
fn different_connectors_on_the_same_device_produce_different_ids() {
    let first_connector = output_id_for_connector(42, 10);
    let second_connector = output_id_for_connector(42, 11);

    assert_ne!(first_connector, second_connector);
}

#[test]
fn supports_maximum_device_and_connector_identifiers() {
    let output_id = output_id_for_connector(u64::MAX, u32::MAX);

    assert_eq!(
        output_id,
        (u128::from(u64::MAX) << 32) | u128::from(u32::MAX)
    );
}

#[test]
fn output_id_preserves_device_and_connector_components() {
    let device_id = 0x1234_5678_9ABC_DEF0;
    let connector_id = 0x1234_5678;

    let output_id = output_id_for_connector(device_id, connector_id);

    assert_eq!((output_id >> 32) as u64, device_id);
    assert_eq!(output_id as u32, connector_id);
}
