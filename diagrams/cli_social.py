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
    "Cli Social Command",
    filename="daily-bugle-cli-social-command",
    direction="TB",
    graph_attr=graph_attr,
):
    request = User("Request Prompt")

    content_intent_service = Preparation("Content Intent Service")

    with Cluster("Workers"):
        content_parsers = [Action(f"Analyzer_{n}") for n in range(1, 4)]

    timeout = Storage("Timeout")
    social_storage = Database("Social Storage")
    timeout >> content_parsers >> social_storage

    reminder = Display("Reminders") >> Edge(label="Feeds") << content_intent_service
    weather = Display("Weather") >> Edge(label="Feeds") << content_intent_service

    (request >> content_intent_service >> Edge(label="Searches") << social_storage)
