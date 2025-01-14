from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import AppsStub
from ..protocol.bridge_pb2 import (
    ID,
    DeployAppRequest,
    DockerImage,
    DockerApp,
    Empty,
    ListRunningAppsRequest,
    Peer,
    RunningApp,
    StopAppRequest,
)

from typing import Iterable, Optional


class AppsService:
    """
    A handle to the application management service.

    Exposes methods to interact with the application management service,
    like for deploying and stopping apps on peers in the network.
    """

    def __init__(self, conn: Channel):
        self.stub = AppsStub(conn)
        self.empty = Empty()

    async def deploy(
        self,
        image: str,
        local: bool,
        ports: Iterable[int] = [],
        peer_id: Optional[str] = None,
    ) -> str:
        """
        Deploys an application in a docker image to a peer in the network.

        Returns the ULID of the deployed application.

        To deploy to self, leave the peer_id argument empty.

        Parameters
        ----------
        image : str
            The name of the docker image to deploy
        local : bool
            Whether the image is available locally
        ports : Iterable[int], optional
            Ports to expose on the container (default: [])
        peer_id : str, optional
            The peer_id of the target node or None to deploy to self (default: None)

        Returns
        -------
        app_id : str
            The id of the deployed application
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        id = await self.stub.Deploy(
            DeployAppRequest(
                app=DockerApp(image=DockerImage(name=image), ports=ports),
                local=local,
                peer=peer,
            )
        )
        return id.ulid

    async def list_running(self, peer_id: Optional[str] = None) -> Iterable[RunningApp]:
        """
        Lists the running apps on a peer in the network.

        To list the running apps on self, leave the peer_id argument empty.

        Parameters
        ----------
        peer_id : str, optional
            The peer_id of the target node or None to list apps on self (default: None)

        Returns
        -------
        app_ids : Iterable[str]
            The ids of the running applications
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        response = await self.stub.ListRunning(ListRunningAppsRequest(peer=peer))
        return response.apps

    async def stop(self, app_id: str, peer_id: Optional[str] = None):
        """
        Stops a running app with an ID on a peer in the network.

        To stop the running app on self, leave the peer_id argument empty.

        Parameters
        ----------
        app_id : str
            The id of the app to stop
        peer_id : str, optional
            The peer_id of the target node or None to stop the app on self (default: None)
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        await self.stub.Stop(StopAppRequest(id=ID(ulid=app_id), peer=peer))

    async def get_own_app_id(self) -> str:
        """
        Get the ID of the current app.

        This can only be called from a running app.

        Returns
        -------
        app_id : str
            The id of the current app
        """

        id = await self.stub.GetOwnAppId(self.empty)
        return id.ulid
