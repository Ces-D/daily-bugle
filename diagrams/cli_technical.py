from diagrams import Cluster, Diagram, Edge, Node
from diagrams.generic.storage import Storage
from diagrams.onprem.client import User
from diagrams.programming.flowchart import (
    Action,
    Database,
    Decision,
    Display,
    MultipleDocuments,
    Preparation,
    StoredData,
)

graph_attr = {"bgcolor": "white"}


with Diagram(
    "Cli Technical Command",
    filename="daily-bugle-cli-technical-command",
    direction="TB",
    graph_attr=graph_attr,
):
    request = User("Request Prompt")

    content_intent_service = Preparation("Content Intent Service")

    with Cluster("Workers"):
        content_parsers = [Action(f"Analyzer_{n}") for n in range(1, 4)]

    technical_storage = Database("Technical Storage")
    technical_sources = MultipleDocuments("Technical Sources")
    technical_sources >> content_parsers >> technical_storage

    (request >> content_intent_service >> Edge(label="Searches") << technical_storage)
