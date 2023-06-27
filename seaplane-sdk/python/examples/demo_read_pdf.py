from seaplane import app, context, log, start, task
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)


@task(type="vectordb", id="save-pdfs")
def save_pdfs(input, store):
    print(store)

    filename = input["filename"].lower().replace(" ", "_")
    store.save(filename, input["file"])

    return input


@task(type="vectordb", id="query-pdfs")
def query_pdfs(input, store):
    print(input)

    return store.fetch_all(input["filename"], input["query"])


@app(path="/save_pdfs", method="POST", parameters=["files"], id="save_files")
def save_files(input):
    print(input)
    return save_pdfs(input)


@app(path="/query_pdfs", method="POST", id="query_pdfs")
def save_files(input):

    return query_pdfs(input)


app = start()
