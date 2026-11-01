use std::io;

use io::BufRead;
use io::Read;

use rs_bat2sqlite::arrow_array;
use rs_bat2sqlite::arrow_schema;
use rs_bat2sqlite::rusqlite;

use rusqlite::Connection;
use rusqlite::Statement;
use rusqlite::Transaction;

use arrow_array::RecordBatch;

use arrow_schema::SchemaRef;

use arrow_ipc::reader::StreamReader;

use rs_bat2sqlite::Converter;

pub fn strm2sqlite<R>(
    strm: StreamReader<R>,
    con: &mut Connection,
    inst: &str,
) -> Result<usize, io::Error>
where
    R: Read,
{
    let sref: SchemaRef = strm.schema();
    let convs: Vec<Box<Converter>> = rs_bat2sqlite::schema2converters(&sref)?;
    let conv = rs_bat2sqlite::converters2converter(convs);
    let mut tot: usize = 0;
    for rslt in strm {
        let tx: Transaction = con.transaction().map_err(io::Error::other)?;
        let mut stmt: Statement = tx.prepare(inst).map_err(io::Error::other)?;
        let rbat: RecordBatch = rslt.map_err(io::Error::other)?;
        let chg: usize = rs_bat2sqlite::bat2sqlite(&rbat, &mut stmt, &conv)?;
        stmt.finalize().map_err(io::Error::other)?;
        tx.commit().map_err(io::Error::other)?;
        tot += chg;
    }
    Ok(tot)
}

pub struct Config {
    pub create_sql: String,
    pub insert_sql: String,
    pub path2sqlite: String,
}

impl Config {
    pub fn brdr2ipc2sqlite<R>(&self, brdr: R) -> Result<usize, io::Error>
    where
        R: BufRead,
    {
        let strm: StreamReader<_> = StreamReader::try_new(brdr, None).map_err(io::Error::other)?;
        let mut con: Connection = Connection::open(&self.path2sqlite).map_err(io::Error::other)?;
        con.execute(&self.create_sql, ())
            .map_err(io::Error::other)?;
        let tot: usize = strm2sqlite(strm, &mut con, &self.insert_sql)?;
        con.close().map_err(|p| p.1).map_err(io::Error::other)?;
        Ok(tot)
    }
}

impl Config {
    pub fn stdin2ipc2sqlite(&self) -> Result<usize, io::Error> {
        self.brdr2ipc2sqlite(io::stdin().lock())
    }
}
