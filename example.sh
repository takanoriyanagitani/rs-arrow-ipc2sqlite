#!/bin/bash

set -u

wsm="./arrow-ipc2sqlite.wasm"

hdir="$PWD"
gdir="/guest.d"
oname="out.sqlite.db3"

hpath="${hdir}/${oname}"
gpath="${gdir}/${oname}"

istrm="./input.arrow.ipc.stream.dat"

create="
  CREATE TABLE IF NOT EXISTS logs(
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    severity TEXT NOT NULL,
    body TEXT NOT NULL
  )
"

insert="
  INSERT INTO logs(timestamp,severity,body)
  VALUES(?1,?2,?3)
"

run_wasmtime(){
  cat /dev/stdin |
    wasmtime \
      run \
      --env ENV_CREATE_SQL="${create}" \
      --env ENV_INSERT_SQL="${insert}" \
      --env ENV_SQLITE_DB_FILE_PATH="${gpath}" \
      --dir "${hdir}::${gdir}" \
      "${wsm}"
}

run_wasi(){
  run_wasmtime
}

icsv(){
  echo timestamp,severity,body
  echo 2,INFO,apt update done
  echo 3,WARN,apt update timeout
  echo 5,ERROR,apt not found
  echo 7,FATAL,kernel panic
}

geninput(){
  echo generating input arrow ipc stream file...

  imports=""
  imports="${imports}import functools;"
  imports="${imports}import sys;"
  imports="${imports}import pyarrow as pa;"
  imports="${imports}import pyarrow.csv as pc;"
  icsv |
    python3 -c "$imports"'functools.reduce(
      lambda state,f: f(state),
      [
        pc.open_csv,
        lambda rdr: (
          pa.ipc.new_stream(sys.stdout.buffer, rdr.schema),
          rdr,
        ),
        lambda pair: map(
          pair[0].write_batch,
          pair[1],
        ),
        lambda writes: sum(1 for _ in writes),
        lambda _: sys.stdout.buffer.flush(),
      ],
      sys.stdin.buffer,
    )' |
    cat > "${istrm}"
}

test -f "${istrm}" || geninput

echo converting...
cat "${istrm}" | run_wasi
echo

echo converted:
echo "SELECT * FROM logs" | sqlite3 -markdown "${hpath}"
echo
