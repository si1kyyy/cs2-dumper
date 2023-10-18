use convert_case::{Case, Casing};

use crate::builder::FileBuilderEnum;
use crate::dumpers::Entry;
use crate::error::Result;
use crate::remote::Process;
use crate::sdk::InterfaceReg;

use super::{generate_files, Entries};

pub fn dump_interfaces(builders: &mut Vec<FileBuilderEnum>, process: &Process) -> Result<()> {
    let mut entries = Entries::new();

    for module_name in process
        .get_loaded_modules()?
        .into_iter()
        .filter(|module_name| *module_name != "crashhandler64.dll")
    {
        let module = process.get_module_by_name(&module_name)?;

        if let Some(create_interface_export) = module.export("CreateInterface") {
            log::info!("Dumping interfaces in {}...", module_name);

            let create_interface_address =
                process.resolve_rip(create_interface_export.va, None, None)?;

            if let Ok(mut interface_reg) =
                process.read_memory::<*mut InterfaceReg>(create_interface_address)
            {
                while !interface_reg.is_null() {
                    let ptr = unsafe { (*interface_reg).ptr(process) }?;
                    let name = unsafe { (*interface_reg).name(process) }?;

                    log::debug!(
                        "  └─ {} @ {:#X} ({} + {:#X})",
                        name,
                        ptr,
                        module_name,
                        ptr - module.base()
                    );

                    entries
                        .entry(
                            module_name
                                .replace(".", "_")
                                .to_case(Case::Snake)
                                .to_case(Case::Pascal),
                        )
                        .or_default()
                        .data
                        .push(Entry {
                            name: name.clone(),
                            value: ptr - module.base(),
                            comment: None,
                        });

                    interface_reg = unsafe { (*interface_reg).next(process) }?;
                }
            }
        }
    }

    generate_files(builders, &entries, "interfaces")?;

    Ok(())
}
