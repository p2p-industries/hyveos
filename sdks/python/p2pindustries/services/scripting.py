from grpc.aio import Channel
from ..protocol.script_pb2_grpc import ScriptingStub
from ..protocol.script_pb2 import (
    ID,
    DeployScriptRequest,
    DockerImage,
    DockerScript,
    Empty,
    ListRunningScriptsRequest,
    Peer,
    RunningScript,
    StopScriptRequest,
)

from typing import Iterable, Optional


class ScriptingService:
    """
    Exposes methods for managing scripts running on nodes in the network
    """

    def __init__(self, conn: Channel):
        self.stub = ScriptingStub(conn)
        self.empty = Empty()

    async def deploy_script(
        self,
        image: str,
        local: bool,
        ports: Iterable[int] = [],
        peer_id: Optional[str] = None,
    ) -> str:
        """
        Deploy a script to a peer and get the id of the deployed script

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
        script_id : str
            The id of the deployed script
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        id = await self.stub.DeployScript(
            DeployScriptRequest(
                script=DockerScript(image=DockerImage(name=image), ports=ports),
                local=local,
                peer=peer,
            )
        )
        return id.ulid

    async def list_running_scripts(
        self, peer_id: Optional[str] = None
    ) -> Iterable[RunningScript]:
        """
        List running scripts on a peer

        Parameters
        ----------
        peer_id : str, optional
            The peer_id of the target node or None to list scripts on self (default: None)

        Returns
        -------
        script_ids : Iterable[str]
            The ids of the running scripts
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        response = await self.stub.ListRunningScripts(
            ListRunningScriptsRequest(peer=peer)
        )
        return response.scripts

    async def stop_script(self, script_id: str, peer_id: Optional[str] = None):
        """
        Stop a running script on a peer

        Parameters
        ----------
        script_id : str
            The id of the script to stop
        peer_id : str, optional
            The peer_id of the target node or None to stop the script on self (default: None)
        """

        if peer_id is not None:
            peer = Peer(peer_id=peer_id)
        else:
            peer = None

        await self.stub.StopScript(StopScriptRequest(id=ID(ulid=script_id), peer=peer))

    async def get_own_id(self) -> str:
        """
        Get the id of the current script

        Returns
        -------
        id : str
            The id of the current script
        """

        id = await self.stub.GetOwnId(self.empty)
        return id.ulid
