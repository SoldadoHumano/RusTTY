fn main() {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("SoldadoHumano")
        .repo_name("RusTTY")
        .bin_name("rustty.exe")
        .target("rustty.exe")
        .build().unwrap();
    let latest = updater.get_latest_release().unwrap();
    println!("{}", latest.version);
}
