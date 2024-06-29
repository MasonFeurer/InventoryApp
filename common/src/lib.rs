pub mod inv;
pub mod old_inv;

use inv::{Id, Inv, Item};
use std::collections::HashMap;
use std::io::{Read, Write};

pub const DATA_VERSION: DataVersion = DataVersion(1);
pub const OLD_DATA_VERSION: DataVersion = DataVersion(0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataVersion(pub u8);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Version(pub u8, pub u8, pub u8);

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CmdCode {
    GetVersion = 0,
    GetInv = 1,
    InsertItem = 2,
    RemoveItem = 3,
    GetServerClients = 4,
    CreateServerBackup = 5,
    ConnectionSuccessfull = 10,
    OperationSuccessfull = 11,
    CmdResponseRecieved = 12,
}
impl CmdCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::GetVersion),
            1 => Some(Self::GetInv),
            2 => Some(Self::InsertItem),
            3 => Some(Self::RemoveItem),
            4 => Some(Self::GetServerClients),
            5 => Some(Self::CreateServerBackup),
            10 => Some(Self::ConnectionSuccessfull),
            11 => Some(Self::OperationSuccessfull),
            12 => Some(Self::CmdResponseRecieved),
            _ => None,
        }
    }
}

pub fn expect_code<T: std::io::Read>(io: &mut T, code: CmdCode) -> std::io::Result<()> {
    let mut code_buf = [0u8];
    io.read_exact(&mut code_buf)?;
    if code_buf[0] != code as u8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unexpected code recieved",
        ));
    }
    Ok(())
}
pub fn send_code<T: std::io::Write>(io: &mut T, code: CmdCode) -> std::io::Result<()> {
    io.write_all(&[code as u8])
}

pub type ClientId = u32;

pub struct ServerHost<C> {
    pub version: Version,
    pub clients: HashMap<ClientId, (String, C)>,
    pub inv: Inv,
}
impl<C: Read + Write> ServerHost<C> {
    pub fn new(version: Version, inv: Inv) -> Self {
        let clients = Default::default();
        Self {
            version,
            clients,
            inv,
        }
    }

    pub fn connect_client(&mut self, mut io: C) -> std::io::Result<ClientId> {
        let version = {
            let mut buf = [0u8; 1];
            io.read_exact(&mut buf)?;
            buf[0]
        };
        let name = {
            let mut len_buf = [0u8; 4];
            io.read_exact(&mut len_buf)?;
            let len = u32::from_be_bytes(len_buf);

            let mut buf = vec![0u8; len as usize];
            io.read_exact(&mut buf)?;
            String::from_utf8_lossy(&buf).to_string()
        };
        send_code(&mut io, CmdCode::ConnectionSuccessfull)?;

        let id = fastrand::u32(..);
        self.clients.insert(id, (name.clone(), io));
        println!("Successfully connected client ({name}) {version} {id:?}");
        Ok(id)
    }

    pub fn handle_client_cmd(&mut self, id: ClientId, cmd: CmdCode) -> std::io::Result<()> {
        let (name, io) = self.clients.get_mut(&id).unwrap();
        match cmd {
            CmdCode::GetVersion => io.write_all(&[self.version])?,
            CmdCode::GetInv => {
                let bytes = bincode::serialize(&self.inv).unwrap();
                let len = bytes.len() as u32;
                io.write_all(&len.to_be_bytes())?;
                io.write_all(&bytes)?;
            }
            CmdCode::InsertItem => {
                let mut id_bytes = [0u8; 4];
                io.read_exact(&mut id_bytes)?;
                let id = u32::from_be_bytes(id_bytes);

                let mut size_bytes = [0u8; 4];
                io.read_exact(&mut size_bytes)?;
                let size = u32::from_be_bytes(size_bytes);

                let mut item_bytes = vec![0u8; size as usize];
                io.read_exact(&mut item_bytes)?;

                let Ok(item) = bincode::deserialize(&item_bytes) else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Item data recieved from client ({name:?}) is invalid"),
                    ));
                };
                self.inv.items.insert(Id(id), item);
                send_code(io, CmdCode::OperationSuccessfull)?;
            }
            CmdCode::RemoveItem => {
                let mut id_bytes = [0u8; 4];
                io.read_exact(&mut id_bytes)?;
                let id = u32::from_be_bytes(id_bytes);

                if !self.inv.items.contains_key(&Id(id)) {
                    eprintln!("Client ({name}) tried to remove item with id {id:x}, but it does not exist.");
                }
                self.inv.items.remove(&Id(id));

                send_code(io, CmdCode::OperationSuccessfull)?;
            }
            CmdCode::GetServerClients => {
                send_code(io, CmdCode::OperationSuccessfull)?;
            }
            CmdCode::CreateServerBackup => {
                send_code(io, CmdCode::OperationSuccessfull)?;
            }
            code => {
                eprintln!("Recieved unexpected command code from client ({name}) : {code:?}")
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ServerErr {
    TimedOut,
    OtherIo(std::io::Error),
    IncompatibleDataVersion(DataVersion),
}
impl From<std::io::Error> for ServerErr {
    fn from(err: std::io::Error) -> ServerErr {
        Self::OtherIo(err)
        // match err.kind() {
        // 	std::io::ErrorKind::
        // }
    }
}

pub struct ServerConn<T> {
    io: T,
    pub supported_data_ver: DataVersion,
}
impl<T: Read + Write> ServerConn<T> {
    pub fn connect(
        mut io: T,
        name: &str,
        supported_data_ver: DataVersion,
    ) -> Result<Self, ServerErr> {
        eprintln!("ServerConn::connect running");
        io.write_all(&inv_version.to_be_bytes())?;
        io.write_all(&(name.len() as u32).to_be_bytes())?;
        io.write_all(name.as_bytes())?;

        expect_code(&mut io, CmdCode::ConnectionSuccessfull)?;
        eprintln!("ServerConn::connect finished");
        Ok(Self {
            io,
            supported_data_ver,
        })
    }

    pub fn get_data_version(&mut self) -> Result<DataVersion, ServerErr> {
        send_code(&mut self.io, CmdCode::GetDataVersion)?;

        let mut buf = [0u8; 1];
        self.io.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn get_version(&mut self) -> Result<Version, ServerErr> {
        send_code(&mut self.io, CmdCode::GetVersion)?;

        let mut buf = [0u8; 1];
        self.io.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn get_inv(&mut self) -> Result<Inv, ServerErr> {
        // Check inv version first
        if self.get_inv_version()? != self.inv_version {
            return Err(ServerErr::IncompatibleVersion);
        }

        eprintln!("ServerConn::get_inv running");
        send_code(&mut self.io, CmdCode::GetInv)?;

        let mut len_bytes = [0u8; 4];
        self.io.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes);

        let mut inv_bytes = vec![0u8; len as usize];
        self.io.read_exact(&mut inv_bytes)?;
        let inv: Inv = bincode::deserialize(&inv_bytes)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        eprintln!("ServerConn::get_inv finished");
        Ok(inv)
    }

    pub fn insert_item(&mut self, id: Id, item: &Item) -> std::io::Result<()> {
        eprintln!("ServerConn::insert_item running");
        let item_bytes = bincode::serialize(item).unwrap();

        send_code(&mut self.io, CmdCode::InsertItem)?;

        self.io.write_all(&id.0.to_be_bytes())?;
        self.io
            .write_all(&(item_bytes.len() as u32).to_be_bytes())?;
        self.io.write_all(&item_bytes)?;

        expect_code(&mut self.io, CmdCode::OperationSuccessfull)?;
        eprintln!("ServerConn::insert_item finished");
        Ok(())
    }

    pub fn remove_item(&mut self, id: Id) -> std::io::Result<()> {
        eprintln!("ServerConn::remove_item running");
        send_code(&mut self.io, CmdCode::RemoveItem)?;
        self.io.write_all(&id.0.to_be_bytes())?;

        expect_code(&mut self.io, CmdCode::OperationSuccessfull)?;
        eprintln!("ServerConn::remove_item finished");
        Ok(())
    }
}
