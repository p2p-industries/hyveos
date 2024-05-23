# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc
import warnings

import script_pb2 as script__pb2

GRPC_GENERATED_VERSION = '1.63.0'
GRPC_VERSION = grpc.__version__
EXPECTED_ERROR_RELEASE = '1.65.0'
SCHEDULED_RELEASE_DATE = 'June 25, 2024'
_version_not_supported = False

try:
    from grpc._utilities import first_version_is_lower
    _version_not_supported = first_version_is_lower(GRPC_VERSION, GRPC_GENERATED_VERSION)
except ImportError:
    _version_not_supported = True

if _version_not_supported:
    warnings.warn(
        f'The grpc package installed is at version {GRPC_VERSION},'
        + f' but the generated code in script_pb2_grpc.py depends on'
        + f' grpcio>={GRPC_GENERATED_VERSION}.'
        + f' Please upgrade your grpc module to grpcio>={GRPC_GENERATED_VERSION}'
        + f' or downgrade your generated code using grpcio-tools<={GRPC_VERSION}.'
        + f' This warning will become an error in {EXPECTED_ERROR_RELEASE},'
        + f' scheduled for release on {SCHEDULED_RELEASE_DATE}.',
        RuntimeWarning
    )


class ReqRespStub(object):
    """----- SERVICES -----

    """

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Send = channel.unary_unary(
                '/script.ReqResp/Send',
                request_serializer=script__pb2.SendRequest.SerializeToString,
                response_deserializer=script__pb2.Response.FromString,
                _registered_method=True)
        self.Recv = channel.unary_stream(
                '/script.ReqResp/Recv',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.RecvRequest.FromString,
                _registered_method=True)
        self.Respond = channel.unary_unary(
                '/script.ReqResp/Respond',
                request_serializer=script__pb2.SendResponse.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.Subscribe = channel.unary_unary(
                '/script.ReqResp/Subscribe',
                request_serializer=script__pb2.Topic.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.Unsubscribe = channel.unary_unary(
                '/script.ReqResp/Unsubscribe',
                request_serializer=script__pb2.Topic.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)


class ReqRespServicer(object):
    """----- SERVICES -----

    """

    def Send(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Recv(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Respond(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Subscribe(self, request, context):
        """Multiple scripts on one node:
        discriminate listening/sending based on Topics
        """
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Unsubscribe(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_ReqRespServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'Send': grpc.unary_unary_rpc_method_handler(
                    servicer.Send,
                    request_deserializer=script__pb2.SendRequest.FromString,
                    response_serializer=script__pb2.Response.SerializeToString,
            ),
            'Recv': grpc.unary_stream_rpc_method_handler(
                    servicer.Recv,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.RecvRequest.SerializeToString,
            ),
            'Respond': grpc.unary_unary_rpc_method_handler(
                    servicer.Respond,
                    request_deserializer=script__pb2.SendResponse.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'Subscribe': grpc.unary_unary_rpc_method_handler(
                    servicer.Subscribe,
                    request_deserializer=script__pb2.Topic.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'Unsubscribe': grpc.unary_unary_rpc_method_handler(
                    servicer.Unsubscribe,
                    request_deserializer=script__pb2.Topic.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.ReqResp', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class ReqResp(object):
    """----- SERVICES -----

    """

    @staticmethod
    def Send(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.ReqResp/Send',
            script__pb2.SendRequest.SerializeToString,
            script__pb2.Response.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Recv(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(
            request,
            target,
            '/script.ReqResp/Recv',
            script__pb2.Empty.SerializeToString,
            script__pb2.RecvRequest.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Respond(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.ReqResp/Respond',
            script__pb2.SendResponse.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Subscribe(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.ReqResp/Subscribe',
            script__pb2.Topic.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Unsubscribe(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.ReqResp/Unsubscribe',
            script__pb2.Topic.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)


class DiscoveryStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.SubscribeEvents = channel.unary_stream(
                '/script.Discovery/SubscribeEvents',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.NeighbourEvent.FromString,
                _registered_method=True)
        self.GetCurrentNeighbours = channel.unary_unary(
                '/script.Discovery/GetCurrentNeighbours',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.Peers.FromString,
                _registered_method=True)
        self.GetOwnId = channel.unary_unary(
                '/script.Discovery/GetOwnId',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.Peer.FromString,
                _registered_method=True)


class DiscoveryServicer(object):
    """Missing associated documentation comment in .proto file."""

    def SubscribeEvents(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def GetCurrentNeighbours(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def GetOwnId(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_DiscoveryServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'SubscribeEvents': grpc.unary_stream_rpc_method_handler(
                    servicer.SubscribeEvents,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.NeighbourEvent.SerializeToString,
            ),
            'GetCurrentNeighbours': grpc.unary_unary_rpc_method_handler(
                    servicer.GetCurrentNeighbours,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.Peers.SerializeToString,
            ),
            'GetOwnId': grpc.unary_unary_rpc_method_handler(
                    servicer.GetOwnId,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.Peer.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.Discovery', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class Discovery(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def SubscribeEvents(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(
            request,
            target,
            '/script.Discovery/SubscribeEvents',
            script__pb2.Empty.SerializeToString,
            script__pb2.NeighbourEvent.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def GetCurrentNeighbours(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.Discovery/GetCurrentNeighbours',
            script__pb2.Empty.SerializeToString,
            script__pb2.Peers.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def GetOwnId(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.Discovery/GetOwnId',
            script__pb2.Empty.SerializeToString,
            script__pb2.Peer.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)


class GossipSubStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Subscribe = channel.unary_unary(
                '/script.GossipSub/Subscribe',
                request_serializer=script__pb2.Topic.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.Unsubscribe = channel.unary_unary(
                '/script.GossipSub/Unsubscribe',
                request_serializer=script__pb2.Topic.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.Publish = channel.unary_unary(
                '/script.GossipSub/Publish',
                request_serializer=script__pb2.Message.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.Recv = channel.unary_stream(
                '/script.GossipSub/Recv',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.GossipSubMessage.FromString,
                _registered_method=True)


class GossipSubServicer(object):
    """Missing associated documentation comment in .proto file."""

    def Subscribe(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Unsubscribe(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Publish(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Recv(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_GossipSubServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'Subscribe': grpc.unary_unary_rpc_method_handler(
                    servicer.Subscribe,
                    request_deserializer=script__pb2.Topic.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'Unsubscribe': grpc.unary_unary_rpc_method_handler(
                    servicer.Unsubscribe,
                    request_deserializer=script__pb2.Topic.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'Publish': grpc.unary_unary_rpc_method_handler(
                    servicer.Publish,
                    request_deserializer=script__pb2.Message.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'Recv': grpc.unary_stream_rpc_method_handler(
                    servicer.Recv,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.GossipSubMessage.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.GossipSub', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class GossipSub(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def Subscribe(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.GossipSub/Subscribe',
            script__pb2.Topic.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Unsubscribe(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.GossipSub/Unsubscribe',
            script__pb2.Topic.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Publish(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.GossipSub/Publish',
            script__pb2.Message.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Recv(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(
            request,
            target,
            '/script.GossipSub/Recv',
            script__pb2.Empty.SerializeToString,
            script__pb2.GossipSubMessage.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)


class DHTStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.PutRecord = channel.unary_unary(
                '/script.DHT/PutRecord',
                request_serializer=script__pb2.DHTRecord.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.GetRecord = channel.unary_unary(
                '/script.DHT/GetRecord',
                request_serializer=script__pb2.DHTKey.SerializeToString,
                response_deserializer=script__pb2.DHTRecord.FromString,
                _registered_method=True)
        self.Provide = channel.unary_unary(
                '/script.DHT/Provide',
                request_serializer=script__pb2.DHTKey.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.GetProviders = channel.unary_stream(
                '/script.DHT/GetProviders',
                request_serializer=script__pb2.DHTKey.SerializeToString,
                response_deserializer=script__pb2.Peer.FromString,
                _registered_method=True)


class DHTServicer(object):
    """Missing associated documentation comment in .proto file."""

    def PutRecord(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def GetRecord(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Provide(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def GetProviders(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_DHTServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'PutRecord': grpc.unary_unary_rpc_method_handler(
                    servicer.PutRecord,
                    request_deserializer=script__pb2.DHTRecord.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'GetRecord': grpc.unary_unary_rpc_method_handler(
                    servicer.GetRecord,
                    request_deserializer=script__pb2.DHTKey.FromString,
                    response_serializer=script__pb2.DHTRecord.SerializeToString,
            ),
            'Provide': grpc.unary_unary_rpc_method_handler(
                    servicer.Provide,
                    request_deserializer=script__pb2.DHTKey.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'GetProviders': grpc.unary_stream_rpc_method_handler(
                    servicer.GetProviders,
                    request_deserializer=script__pb2.DHTKey.FromString,
                    response_serializer=script__pb2.Peer.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.DHT', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class DHT(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def PutRecord(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.DHT/PutRecord',
            script__pb2.DHTRecord.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def GetRecord(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.DHT/GetRecord',
            script__pb2.DHTKey.SerializeToString,
            script__pb2.DHTRecord.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def Provide(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.DHT/Provide',
            script__pb2.DHTKey.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def GetProviders(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(
            request,
            target,
            '/script.DHT/GetProviders',
            script__pb2.DHTKey.SerializeToString,
            script__pb2.Peer.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)


class FileTransferStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.PublishFile = channel.unary_unary(
                '/script.FileTransfer/PublishFile',
                request_serializer=script__pb2.File.SerializeToString,
                response_deserializer=script__pb2.Empty.FromString,
                _registered_method=True)
        self.GetFile = channel.unary_unary(
                '/script.FileTransfer/GetFile',
                request_serializer=script__pb2.ID.SerializeToString,
                response_deserializer=script__pb2.FilePath.FromString,
                _registered_method=True)


class FileTransferServicer(object):
    """Missing associated documentation comment in .proto file."""

    def PublishFile(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def GetFile(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_FileTransferServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'PublishFile': grpc.unary_unary_rpc_method_handler(
                    servicer.PublishFile,
                    request_deserializer=script__pb2.File.FromString,
                    response_serializer=script__pb2.Empty.SerializeToString,
            ),
            'GetFile': grpc.unary_unary_rpc_method_handler(
                    servicer.GetFile,
                    request_deserializer=script__pb2.ID.FromString,
                    response_serializer=script__pb2.FilePath.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.FileTransfer', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class FileTransfer(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def PublishFile(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.FileTransfer/PublishFile',
            script__pb2.File.SerializeToString,
            script__pb2.Empty.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def GetFile(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/script.FileTransfer/GetFile',
            script__pb2.ID.SerializeToString,
            script__pb2.FilePath.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)


class DebugStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.SubscribeMeshTopology = channel.unary_stream(
                '/script.Debug/SubscribeMeshTopology',
                request_serializer=script__pb2.Empty.SerializeToString,
                response_deserializer=script__pb2.MeshTopologyEvent.FromString,
                _registered_method=True)


class DebugServicer(object):
    """Missing associated documentation comment in .proto file."""

    def SubscribeMeshTopology(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_DebugServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'SubscribeMeshTopology': grpc.unary_stream_rpc_method_handler(
                    servicer.SubscribeMeshTopology,
                    request_deserializer=script__pb2.Empty.FromString,
                    response_serializer=script__pb2.MeshTopologyEvent.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'script.Debug', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class Debug(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def SubscribeMeshTopology(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(
            request,
            target,
            '/script.Debug/SubscribeMeshTopology',
            script__pb2.Empty.SerializeToString,
            script__pb2.MeshTopologyEvent.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)
