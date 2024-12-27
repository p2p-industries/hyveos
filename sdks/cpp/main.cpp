#include <grpcpp/grpcpp.h>

#include <memory>

#include "script.grpc.pb.h"
#include "script.pb.h"

class ReqRespClient {
   public:
    ReqRespClient(std::shared_ptr<grpc::Channel> channel)
        : stub_(script::ReqResp::NewStub(channel)) {}

   private:
    std::unique_ptr<script::ReqResp::Stub> stub_;
};
