// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS test module relocated from production source.

use super::*;

#[test]
fn derives_stable_ids_from_connector_ids() {
    assert_eq!(output_id_for_connector(1, 10), Ok(11));
    assert_eq!(output_id_for_connector(1, 20), Ok(21));
}

#[test]
fn connector_ids_do_not_depend_on_enumeration_order() {
    let first = output_id_for_connector(1, 42);
    let second = output_id_for_connector(1, 17);

    assert_eq!(first, output_id_for_connector(1, 42));
    assert_eq!(second, output_id_for_connector(1, 17));
}

#[test]
fn allows_maximum_output_id() {
    assert_eq!(output_id_for_connector(u64::MAX, 0), Ok(u64::MAX));
}

#[test]
fn rejects_output_id_overflow() {
    assert_eq!(
        output_id_for_connector(u64::MAX, 1),
        Err(DrmDiscoveryError::OutputIdOverflow)
    );
}
