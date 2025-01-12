
use dup_file_remover::utils;

#[test]
fn check_ipv6() {
    let ipv6_available = utils::network::check_ipv6_available();
    assert!(ipv6_available);
}
