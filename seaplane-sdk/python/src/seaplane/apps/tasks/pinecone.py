import os
from typing import Any, Callable, Dict, List, Optional, Tuple

from langchain.chains import ConversationalRetrievalChain
from langchain.document_loaders import PyPDFLoader
from langchain.embeddings.openai import OpenAIEmbeddings
from langchain.llms import OpenAI
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain.vectorstores import Pinecone
import pinecone

from ...configuration import config
from ...logging import log
from ...model.errors import SeaplaneError

_PINECONE_INDEX_NAME = "PINECONE_INDEX_ENV"
_PINECONE_API_KEY_NAME = "PINECONE_API_KEY"
_PINECONE_API_ENV_NAME = "PINECONE_API_ENV"


def _check_pinecone_api_key() -> None:
    if config._api_keys is None:
        raise SeaplaneError(
            f"Pinecone API Key `{_PINECONE_API_KEY_NAME}` is not set,\
                  use `sea.config.set_api_keys`."
        )
    elif not config._api_keys.get(_PINECONE_API_KEY_NAME, None) or not config._api_keys.get(
        _PINECONE_API_ENV_NAME, None
    ):
        raise SeaplaneError(
            f"Pinecone API Key `{_PINECONE_API_KEY_NAME}` or `{_PINECONE_API_ENV_NAME}`\
                  is not set, use `sea.config.set_api_keys`."
        )


def get_index() -> str:
    index = os.getenv(_PINECONE_INDEX_NAME)

    if index is None:
        return ""

    return index


class Store:
    def __init__(self) -> None:
        _check_pinecone_api_key()

        self.index = get_index()
        self.chat_history_file: Dict[str, List[Tuple[str, str]]] = {}
        self.text_splitter = RecursiveCharacterTextSplitter(chunk_size=1000, chunk_overlap=0)
        self.embeddings = OpenAIEmbeddings(  # type: ignore
            openai_api_key=config._api_keys.get("OPENAI_API_KEY", None)
        )
        pinecone.init(
            api_key=config._api_keys.get(_PINECONE_API_KEY_NAME, None),
            environment=config._api_keys.get(_PINECONE_API_ENV_NAME, None),
        )

    def save(self, file_name: str, file_url: str) -> None:
        loader = PyPDFLoader(file_url)
        document = loader.load()
        texts = self.text_splitter.split_documents(document)

        log.info(f"⏳ Saving file {file_name}")
        Pinecone.from_texts(
            [chunk.page_content for chunk in texts],
            self.embeddings,
            index_name=self.index,
            namespace=file_name,
        )

    def query(self, file_name: str, query: str) -> Dict[str, Any]:
        vectorstore = Pinecone.from_existing_index(
            index_name=self.index, embedding=self.embeddings, namespace=file_name
        )
        qa = ConversationalRetrievalChain.from_llm(
            llm=OpenAI(  # type: ignore
                model="gpt-3.5-turbo",
                temperature=0.7,
                openai_api_key=config._api_keys.get("OPENAI_API_KEY", None),
            ),
            retriever=vectorstore.as_retriever(),
            return_source_documents=True,
        )

        history = self.chat_history_file.get(file_name, None)
        if history is None:
            history = []
            self.chat_history_file[file_name] = []

        result = qa({"question": query, "chat_history": history})
        self.chat_history_file[file_name].append((query, result["answer"]))

        return {"answer": result["answer"], "history": history}


class PineconeTask:
    def __init__(self, func: Callable[[Any], Any], id: str, model: Optional[str]) -> None:
        self.func = func
        self.args: Optional[Tuple[Any, ...]] = None
        self.kwargs: Optional[Dict[str, Any]] = None
        self.type = "pinecone"
        self.model = model
        self.id = id

    def process(self, *args: Any, **kwargs: Any) -> Any:
        self.args = args
        self.kwargs = kwargs

        if self.type == "pinecone":
            log.info("Accessing Vector DB task...")
            self.args = self.args + (Store(),)

            return self.func(*self.args, **self.kwargs)

    def print(self) -> None:
        log.info(f"id: {self.id}, type: {self.type}, model: {self.model}")
