// SPDX-License-Identifier: Apache-2.0

use crate::{
    unit_tests::testlib::{
        bond_with_ports, bridge_with_ports, new_br_iface, new_eth_iface,
        new_nested_4_ifaces, new_ovs_br_iface, new_ovs_iface,
    },
    ErrorKind, Interface, InterfaceState, InterfaceType, Interfaces,
    MergedInterfaces, OvsBridgeInterface,
};

#[test]
fn test_ifaces_up_order_no_ctrler_reserse_order() {
    let mut cur_ifaces = Interfaces::new();
    cur_ifaces.push(new_eth_iface("eth1"));
    cur_ifaces.push(new_eth_iface("eth2"));
    let mut ifaces = Interfaces::new();
    ifaces.push(new_eth_iface("eth2"));
    ifaces.push(new_eth_iface("eth1"));

    let merged_ifaces =
        MergedInterfaces::new(ifaces, cur_ifaces, false, false).unwrap();

    let eth1_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();
    let eth2_iface = merged_ifaces
        .get_iface("eth2", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(eth1_iface.base_iface().up_priority, 0);
    assert_eq!(eth2_iface.base_iface().up_priority, 0);
}

fn gen_test_eth_ifaces() -> Interfaces {
    let mut ifaces = Interfaces::new();
    ifaces.push(new_eth_iface("p1"));
    ifaces.push(new_eth_iface("p2"));
    ifaces
}

#[test]
fn test_ifaces_up_order_nested_4_depth_worst_case() {
    let mut ifaces = Interfaces::new();

    let [br0, br1, br2, br3, p1, p2] = new_nested_4_ifaces();

    // Push with reverse order which is the worst case
    ifaces.push(p2);
    ifaces.push(p1);
    ifaces.push(br3);
    ifaces.push(br2);
    ifaces.push(br1);
    ifaces.push(br0);

    let merged_ifaces =
        MergedInterfaces::new(ifaces, gen_test_eth_ifaces(), false, false)
            .unwrap();

    assert_eq!(
        merged_ifaces.kernel_ifaces["br0"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        0
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br1"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        1
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br2"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        2
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br3"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        3
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["p1"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        4
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["p2"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        4
    );
}

#[test]
fn test_ifaces_up_order_nested_5_depth_worst_case() {
    let mut ifaces = Interfaces::new();
    let [_, br1, br2, br3, p1, p2] = new_nested_4_ifaces();

    let br4 = new_br_iface("br4");
    let mut br0 = new_br_iface("br0");

    br0.base_iface_mut().controller = Some("br4".to_string());
    br0.base_iface_mut().controller_type = Some(InterfaceType::LinuxBridge);

    // Push with reverse order which is the worst case
    ifaces.push(p1);
    ifaces.push(p2);
    ifaces.push(br3);
    ifaces.push(br2);
    ifaces.push(br1);
    ifaces.push(br0);
    ifaces.push(br4);

    let result =
        MergedInterfaces::new(ifaces, gen_test_eth_ifaces(), false, false);

    assert!(result.is_err());

    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_ifaces_up_order_nested_5_depth_good_case() {
    let mut ifaces = Interfaces::new();
    let [_, br1, br2, br3, p1, p2] = new_nested_4_ifaces();

    let br4 = new_br_iface("br4");
    let mut br0 = new_br_iface("br0");

    br0.base_iface_mut().controller = Some("br4".to_string());
    br0.base_iface_mut().controller_type = Some(InterfaceType::LinuxBridge);

    ifaces.push(br4);
    ifaces.push(br0);
    ifaces.push(br1);
    ifaces.push(br2);
    ifaces.push(br3);
    ifaces.push(p2);
    ifaces.push(p1);

    let merged_ifaces =
        MergedInterfaces::new(ifaces, gen_test_eth_ifaces(), false, false)
            .unwrap();

    assert_eq!(
        merged_ifaces.kernel_ifaces["br4"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        0
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br0"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        1
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br1"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        2
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br2"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        3
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["br3"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        4
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["p1"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        5
    );
    assert_eq!(
        merged_ifaces.kernel_ifaces["p2"]
            .for_apply
            .as_ref()
            .unwrap()
            .base_iface()
            .up_priority,
        5
    );
}

#[test]
fn test_auto_include_ovs_interface() {
    let mut ifaces = Interfaces::new();
    ifaces.push(new_ovs_br_iface("br0", &["p1", "p2"]));

    let merged_ifaces =
        MergedInterfaces::new(ifaces, Interfaces::new(), false, false).unwrap();

    let p1_iface = merged_ifaces
        .get_iface("p1", InterfaceType::OvsInterface)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let p2_iface = merged_ifaces
        .get_iface("p2", InterfaceType::OvsInterface)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();
    let br0_iface = merged_ifaces
        .get_iface("br0", InterfaceType::OvsBridge)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(p1_iface.base_iface().up_priority, 1);
    assert_eq!(p1_iface.base_iface().name, "p1");
    assert_eq!(
        p1_iface.base_iface().iface_type,
        InterfaceType::OvsInterface
    );
    assert_eq!(p1_iface.base_iface().controller, Some("br0".to_string()));
    assert_eq!(
        p1_iface.base_iface().controller_type,
        Some(InterfaceType::OvsBridge)
    );
    assert_eq!(p2_iface.base_iface().up_priority, 1);
    assert_eq!(p2_iface.base_iface().name, "p2");
    assert_eq!(
        p2_iface.base_iface().iface_type,
        InterfaceType::OvsInterface
    );
    assert_eq!(p2_iface.base_iface().controller, Some("br0".to_string()));
    assert_eq!(
        p2_iface.base_iface().controller_type,
        Some(InterfaceType::OvsBridge)
    );
    assert_eq!(br0_iface.base_iface().up_priority, 0);
}

#[test]
fn test_auto_absent_ovs_interface() {
    let mut cur_ifaces = Interfaces::new();
    cur_ifaces.push(new_ovs_br_iface("br0", &["p1", "p2"]));
    cur_ifaces.push(new_ovs_iface("p1", "br0"));
    cur_ifaces.push(new_ovs_iface("p2", "br0"));

    let mut absent_br0 = OvsBridgeInterface::new();
    absent_br0.base.name = "br0".to_string();
    absent_br0.base.state = InterfaceState::Absent;
    let mut ifaces = Interfaces::new();
    ifaces.push(Interface::OvsBridge(absent_br0));

    let merged_ifaces =
        MergedInterfaces::new(ifaces, cur_ifaces, false, false).unwrap();

    let p1_iface = merged_ifaces
        .get_iface("p1", InterfaceType::OvsInterface)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let p2_iface = merged_ifaces
        .get_iface("p2", InterfaceType::OvsInterface)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();
    let br0_iface = merged_ifaces
        .get_iface("br0", InterfaceType::OvsBridge)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(p1_iface.base_iface().name, "p1");
    assert_eq!(
        p1_iface.base_iface().iface_type,
        InterfaceType::OvsInterface
    );
    assert_eq!(p1_iface.base_iface().state, InterfaceState::Absent);

    assert_eq!(p2_iface.base_iface().name, "p2");
    assert_eq!(
        p2_iface.base_iface().iface_type,
        InterfaceType::OvsInterface
    );
    assert_eq!(p2_iface.base_iface().state, InterfaceState::Absent);
    assert_eq!(br0_iface.base_iface().state, InterfaceState::Absent);
}

#[test]
fn test_overbook_port_in_single_bridge() {
    let mut desired = Interfaces::new();

    desired.push(bridge_with_ports("br0", &["eth0"]));
    desired.push(new_eth_iface("eth0"));

    let mut current = Interfaces::new();
    current.push(new_eth_iface("eth0"));

    assert!(MergedInterfaces::new(desired, current, false, false).is_ok());
}

#[test]
fn test_overbook_port_in_two_bridges() {
    let mut desired = Interfaces::new();

    desired.push(bridge_with_ports("br0", &["eth0"]));
    desired.push(bridge_with_ports("br1", &["eth0"]));

    let mut current = Interfaces::new();
    current.push(new_eth_iface("eth0"));

    let result = MergedInterfaces::new(desired, current, false, false);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().kind(), ErrorKind::InvalidArgument);
}

#[test]
fn test_overbook_port_moves_between_bridges() {
    let mut current = Interfaces::new();
    current.push(bridge_with_ports("br0", &["eth0"]));
    let mut eth0 = new_eth_iface("eth0");
    eth0.base_iface_mut().controller = Some("br0".to_string());
    eth0.base_iface_mut().controller_type = Some(InterfaceType::LinuxBridge);
    current.push(eth0);

    let mut desired = Interfaces::new();
    desired.push(bridge_with_ports("br0", &[]));
    desired.push(bridge_with_ports("br1", &["eth0"]));

    assert!(MergedInterfaces::new(desired, current, false, false).is_ok());
}

#[test]
fn test_overbook_current_bridge_is_deleted() {
    let mut current = Interfaces::new();
    current.push(bridge_with_ports("br0", &["eth0"]));
    current.push(new_eth_iface("eth0"));

    let mut desired = Interfaces::new();
    desired.push(bridge_with_ports("br1", &["eth0"]));
    let mut absent_iface = new_br_iface("br0");
    absent_iface.base_iface_mut().state = InterfaceState::Absent;
    desired.push(absent_iface);

    MergedInterfaces::new(desired, current, false, false).unwrap();
}

#[test]
fn test_overbook_port_used_in_current_bridge() {
    let mut current = Interfaces::new();
    current.push(bridge_with_ports("br0", &["eth0"]));
    let mut eth0 = new_eth_iface("eth0");
    eth0.base_iface_mut().controller = Some("br0".to_string());
    eth0.base_iface_mut().controller_type = Some(InterfaceType::LinuxBridge);
    current.push(eth0);

    let mut desired = Interfaces::new();
    desired.push(bridge_with_ports("br1", &["eth0"]));

    assert!(MergedInterfaces::new(desired, current, false, false).is_ok());
}

#[test]
fn test_overbook_port_used_in_current_bond() {
    let mut current = Interfaces::new();
    current.push(bond_with_ports("bond0", &["eth0"]));
    let mut eth0 = new_eth_iface("eth0");
    eth0.base_iface_mut().controller = Some("bond0".to_string());
    eth0.base_iface_mut().controller_type = Some(InterfaceType::Bond);
    current.push(eth0);

    let mut desired = Interfaces::new();
    desired.push(bond_with_ports("bond1", &["eth0"]));

    assert!(MergedInterfaces::new(desired, current, false, false).is_ok());
}

#[test]
fn test_overbook_swap_port_of_bond() {
    let mut current = Interfaces::new();
    current.push(bond_with_ports("bond0", &["eth0"]));
    current.push(bond_with_ports("bond1", &["eth1"]));
    let mut eth0 = new_eth_iface("eth0");
    eth0.base_iface_mut().controller = Some("bond0".to_string());
    eth0.base_iface_mut().controller_type = Some(InterfaceType::Bond);
    current.push(eth0);
    let mut eth1 = new_eth_iface("eth1");
    eth1.base_iface_mut().controller = Some("bond1".to_string());
    eth1.base_iface_mut().controller_type = Some(InterfaceType::Bond);
    current.push(eth1);

    let mut desired = Interfaces::new();
    desired.push(bond_with_ports("bond1", &["eth0"]));
    desired.push(bond_with_ports("bond0", &["eth1"]));

    assert!(MergedInterfaces::new(desired, current, false, false).is_ok());
}

#[test]
fn test_iface_controller_not_conflict_with_bond_ports() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: bond0
  type: bond
  state: up
  link-aggregation:
    mode: balance-rr
  controller: br0
- name: br0
  type: ovs-bridge
  state: up
  bridge:
    port:
      - name: bond0
",
    )
    .unwrap();

    let result =
        MergedInterfaces::new(des_ifaces, Interfaces::new(), false, false);
    assert!(result.is_ok());
}

#[test]
fn test_iface_controller_conflict_with_bond_ports() {
    let mut ifaces = Interfaces::new();
    ifaces.push(new_eth_iface("eth0"));
    ifaces.push(bond_with_ports("bond0", &["eth0"]));
    ifaces.push(bond_with_ports("bond1", &["eth1"]));
    let mut iface = new_eth_iface("eth1");
    iface.base_iface_mut().controller = Some("bond0".to_string());
    ifaces.push(iface);

    let result = MergedInterfaces::new(ifaces, Interfaces::new(), false, false);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_iface_controller_conflict_with_br_ports() {
    let mut cur_ifaces = Interfaces::new();
    cur_ifaces.push(new_eth_iface("eth0"));
    cur_ifaces.push(new_eth_iface("eth1"));
    let mut ifaces = Interfaces::new();
    ifaces.push(new_eth_iface("eth0"));
    ifaces.push(bridge_with_ports("br0", &["eth0"]));
    ifaces.push(bridge_with_ports("br1", &["eth1"]));
    let mut iface = new_eth_iface("eth1");
    iface.base_iface_mut().controller = Some("br0".to_string());
    ifaces.push(iface);

    let result = MergedInterfaces::new(ifaces, cur_ifaces, false, false);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_iface_controller_prop_only_in_desire() {
    let mut current = Interfaces::new();
    current.push(new_eth_iface("eth0"));
    current.push(new_eth_iface("eth1"));
    current.push(bridge_with_ports("br0", &["eth0"]));
    let mut desired = Interfaces::new();
    let mut iface = new_eth_iface("eth1");
    iface.base_iface_mut().controller = Some("br0".to_string());
    desired.push(iface);

    let merged_ifaces =
        MergedInterfaces::new(desired, current, false, false).unwrap();

    let eth1_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(
        eth1_iface.base_iface().controller.as_ref(),
        Some(&"br0".to_string())
    );
    assert_eq!(
        eth1_iface.base_iface().controller_type.as_ref(),
        Some(&InterfaceType::LinuxBridge)
    );
}

#[test]
fn test_iface_controller_prop_only_in_desire_dup_ovs_br() {
    let mut current = Interfaces::new();
    current.push(new_eth_iface("eth0"));
    current.push(new_eth_iface("eth1"));
    current.push(new_ovs_iface("br0", "br0"));
    current.push(new_ovs_br_iface("br0", &["eth0", "eth1", "br0"]));
    let mut desired = Interfaces::new();
    let mut iface = new_eth_iface("eth1");
    iface.base_iface_mut().controller = Some("br0".to_string());
    desired.push(iface);

    let merged_ifaces =
        MergedInterfaces::new(desired, current, false, false).unwrap();

    let eth1_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(
        eth1_iface.base_iface().controller.as_ref(),
        Some(&"br0".to_string())
    );
    assert_eq!(
        eth1_iface.base_iface().controller_type.as_ref(),
        Some(&InterfaceType::OvsBridge)
    );
}

#[test]
fn test_iface_controller_been_list_in_other_port_list() {
    let mut current = Interfaces::new();
    current.push(new_eth_iface("eth0"));
    current.push(bond_with_ports("bond0", &["eth0"]));

    let mut ifaces = Interfaces::new();
    ifaces.push(bond_with_ports("bond1", &["eth1"]));
    let mut iface = new_eth_iface("eth1");
    iface.base_iface_mut().controller = Some("bond0".to_string());
    ifaces.push(iface);

    let result = MergedInterfaces::new(ifaces, current, false, false);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_iface_detach_controller_been_list_in_other_port_list() {
    let mut ifaces = Interfaces::new();
    ifaces.push(bond_with_ports("bond1", &["eth0"]));
    let mut iface = new_eth_iface("eth0");
    iface.base_iface_mut().controller = Some("".to_string());
    ifaces.push(iface);

    let result = MergedInterfaces::new(ifaces, Interfaces::new(), false, false);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_iface_controller_conflict_with_empty_br_ports() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: br0
  type: linux-bridge
  bridge:
    ports: []
- name: bond99
  type: bond
  state: up
  controller: br0
  link-aggregation:
    mode: balance-rr
    port:
    - eth2
    - eth1
",
    )
    .unwrap();

    let result =
        MergedInterfaces::new(des_ifaces, Interfaces::new(), false, false);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_auto_manage_ports() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: br0
  type: linux-bridge
  state: up
  bridge:
    port:
    - name: eth1
    - name: eth2
",
    )
    .unwrap();
    let cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: eth1
  type: ethernet
  state: ignore
- name: eth2
  type: ethernet
  state: ignore
",
    )
    .unwrap();

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let br_iface = merged_ifaces
        .get_iface("br0", InterfaceType::LinuxBridge)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let eth1_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let eth2_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(br_iface.ports(), Some(vec!["eth1", "eth2"]));
    assert!(eth1_iface.is_up());
    assert_eq!(eth1_iface.base_iface().controller.as_deref(), Some("br0"));
    assert_eq!(eth2_iface.base_iface().controller.as_deref(), Some("br0"));
}

#[test]
fn test_auto_manage_ovs_bond_ports() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
        - name: br0
          type: ovs-bridge
          state: up
          bridge:
            port:
            - name: bond1
              link-aggregation:
                mode: balance-slb
                port:
                  - name: eth1
                  - name: eth2",
    )
    .unwrap();
    let cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
        - name: eth1
          type: ethernet
          state: ignore
        - name: eth2
          type: ethernet
          state: ignore",
    )
    .unwrap();

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let br_iface = merged_ifaces
        .get_iface("br0", InterfaceType::OvsBridge)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let eth1_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    let eth2_iface = merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert_eq!(br_iface.ports(), Some(vec!["eth1", "eth2"]));
    assert!(eth1_iface.is_up());
    assert_eq!(eth1_iface.base_iface().controller.as_deref(), Some("br0"));
    assert_eq!(eth2_iface.base_iface().controller.as_deref(), Some("br0"));
}

#[test]
fn test_do_not_auto_manage_ports_if_current_has_ignore() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: br0
  type: linux-bridge
  state: up
  bridge:
    port:
    - name: eth1
    - name: eth2
",
    )
    .unwrap();
    let cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: eth1
  type: ethernet
  state: ignore
- name: eth2
  type: ethernet
  state: ignore
- name: eth3
  type: ethernet
  state: ignore
- name: br0
  type: linux-bridge
  state: up
  bridge:
    port:
    - name: eth3
",
    )
    .unwrap();

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let br_iface = merged_ifaces
        .get_iface("br0", InterfaceType::LinuxBridge)
        .unwrap()
        .for_apply
        .as_ref()
        .unwrap();

    assert!(merged_ifaces
        .get_iface("eth1", InterfaceType::Ethernet)
        .is_none());
    assert!(merged_ifaces
        .get_iface("eth2", InterfaceType::Ethernet)
        .is_none());

    assert_eq!(br_iface.ports(), Some(vec![]));
}

#[test]
fn test_absent_iface_holding_controller_and_ip() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
        - name: eth1
          state: absent",
    )
    .unwrap();

    // Kernel allows NIC hold IP even attached to bridge.
    // When user is removing this interface, we should not care about whether it
    // can hold IP or not.
    let mut cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
        - name: br0
          type: linux-bridge
          state: up
          bridge:
            port:
            - name: eth1
        - name: eth1
          type: ethernet
          state: up
          ipv6:
            enabled: true
            autoconf: true
            dhcp: true",
    )
    .unwrap();
    if let Some(iface) = cur_ifaces.kernel_ifaces.get_mut("eth1") {
        iface.base_iface_mut().controller = Some("br0".to_string());
        iface.base_iface_mut().controller_type =
            Some(InterfaceType::LinuxBridge);
    }

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let iface = merged_ifaces.kernel_ifaces.get("eth1").unwrap();

    assert!(iface.for_apply.as_ref().unwrap().is_absent());
}

#[test]
fn test_gen_topoligies_bridge_over_vlan() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: br0
  type: linux-bridge
  state: up
  bridge:
    port:
    - name: eth1.100
    - name: eth2.100
  ipv4:
    address:
    - ip: 192.0.2.252
      prefix-length: 24
    - ip: 192.0.2.251
      prefix-length: 24
    dhcp: false
    enabled: true
  ipv6:
    autoconf: true
    dhcp: true
    enabled: true
- name: eth1.100
  type: vlan
  vlan:
    id: 100
    base-iface: eth1
- name: eth2.100
  type: vlan
  vlan:
    id: 100
    base-iface: eth2
",
    )
    .unwrap();
    let cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: eth1
  type: ethernet
- name: eth2
  type: ethernet
",
    )
    .unwrap();

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let top = merged_ifaces.gen_topoligies();

    assert_eq!(
        top,
        vec![[
            "static_ip4,auto_ip6".to_string(),
            InterfaceType::LinuxBridge.to_string(),
            InterfaceType::Vlan.to_string(),
            InterfaceType::Ethernet.to_string()
        ]
        .join(" -> ")]
    );
}

#[test]
fn test_gen_topoligies_ovs_bridge() {
    let des_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: br0
  type: ovs-interface
  state: up
  ipv4:
    address:
    - ip: 192.0.2.252
      prefix-length: 24
    - ip: 192.0.2.251
      prefix-length: 24
    dhcp: false
    enabled: true
  ipv6:
    autoconf: true
    dhcp: true
    enabled: true
- name: br0
  type: ovs-bridge
  state: up
  bridge:
    ports:
    - name: br0
    - name: eth1
    - name: eth2
",
    )
    .unwrap();
    let cur_ifaces: Interfaces = serde_yaml::from_str(
        r"---
- name: eth1
  type: ethernet
- name: eth2
  type: ethernet
",
    )
    .unwrap();

    let merged_ifaces =
        MergedInterfaces::new(des_ifaces, cur_ifaces, false, false).unwrap();

    let top = merged_ifaces.gen_topoligies();

    assert_eq!(
        top,
        vec![[
            "static_ip4,auto_ip6".to_string(),
            InterfaceType::OvsInterface.to_string(),
            InterfaceType::OvsBridge.to_string(),
            InterfaceType::Ethernet.to_string()
        ]
        .join(" -> ")]
    );
}
