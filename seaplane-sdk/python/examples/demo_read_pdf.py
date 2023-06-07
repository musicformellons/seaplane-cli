from seaplane import context, coprocessor, log, smartpipe, start
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)


@coprocessor(type="vectordb", id="save-pdfs")
def save_pdfs(input, store):
    print(store)

    filename = input["filename"].lower().replace(" ", "_")
    store.save(filename, input["file"])

    return input


@coprocessor(type="vectordb", id="query-pdfs")
def query_pdfs(input, store):
    print(input)

    return store.query(input["filename"], input["query"])


@smartpipe(path="/save_pdfs", method="POST", parameters=["files"], id="save_files")
def save_files(input):
    print(input)
    return save_pdfs(input)


@smartpipe(path="/query_pdfs", method="POST", id="query_pdfs")
def save_files(input):

    return query_pdfs(input)


app = start()
