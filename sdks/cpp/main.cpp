#include <grpcpp/grpcpp.h>
#include <grpcpp/support/status.h>

#include <memory>
#include <optional>
#include <stdexcept>
#include <string>

#include "script.grpc.pb.h"
#include "script.pb.h"

class ReqRespClient {
   public:
    ReqRespClient(std::shared_ptr<grpc::Channel> channel)
        : stub_(script::ReqResp::NewStub(channel)) {}

    std::string Send(std::string peer_id, std::string data,
                     std::optional<std::string> topic) {
        auto peer = new script::Peer();
        peer->set_allocated_peer_id(&peer_id);

        auto d = new script::Data();
        d->set_allocated_data(&data);

        auto ot = new script::OptionalTopic();
        if (topic.has_value()) {
            auto t = new script::Topic();
            t->set_allocated_topic(&topic.value());
            ot->set_allocated_topic(t);
        }

        auto message = new script::Message();
        message->set_allocated_data(d);
        message->set_allocated_topic(ot);

        script::SendRequest request;
        request.set_allocated_peer(peer);
        request.set_allocated_msg(message);
        script::Response response;

        grpc::ClientContext context;
        grpc::Status status = stub_->Send(&context, request, &response);
        if (status.ok()) {
            auto d = response.data();
            return d.data();
        } else {
            auto error_message = status.error_message();
            throw std::runtime_error(error_message);
        }
    }

   private:
    std::unique_ptr<script::ReqResp::Stub> stub_;
};
