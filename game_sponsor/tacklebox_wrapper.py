import grpclib.server as grpclib_server
import grpclib.const as grpclib_const
from grpclib.exceptions import GRPCError
import rlcard
# 导入由 grpclib 插件生成的文件
import sponsor_pb2 as pb2
import sponsor_grpc as pb2_grpc 
import asyncio
from grpclib.server import Stream
import json 
import numpy 
from typing import AsyncIterable

# 服务器地址和端口配置
HOST = '127.0.0.1'
PORT = 50051

# 继承自 grpclib 插件生成的基类
# **注意：这个基类包含了 grpclib 所需的 __mapping__ 方法**

# Extend JSONEncoder to handle numpy arrays commonly found in rlcard states
class NumpyEncoder(json.JSONEncoder):
    """Custom encoder for numpy data types (like np.int64, np.float32)"""
    def default(self, obj):
        if isinstance(obj, numpy.integer):
            return int(obj)
        elif isinstance(obj, numpy.floating):
            return float(obj)
        elif isinstance(obj, numpy.ndarray):
            # Convert NumPy array to a standard list for JSON
            return obj.tolist()
        return json.JSONEncoder.default(obj)

class SponsorServiceServicer(pb2_grpc.SponsorServiceBase):

    async def ProcessGame(self, stream: Stream):
        print("Game session started.")

        env = None
        is_game_active = False

        await stream.send_message(
            pb2.ProcessGameResponse(
                init_response=pb2.GameInitResponse(type=pb2.GameInitResponse.SUCCESS)
            )
        )

        async for req in stream:
            print(f"Received request: {req}")

            if req.HasField("init"):
                env = rlcard.make(req.init.game_type)
                is_game_active = True

                # 发送 INIT 响应
                await stream.send_message(
                    pb2.ProcessGameResponse(
                        init_response=pb2.GameInitResponse(type=pb2.GameInitResponse.SUCCESS)
                    )
                )

                # 发送初始状态
                state_dict, i_player = env.reset()
                await stream.send_message(
                    pb2.ProcessGameResponse(
                        state_update=pb2.GameStateUpdate(
                            state=json.dumps(state_dict, cls=NumpyEncoder),
                            is_over=False,
                            i_player=i_player,
                        )
                    )
                )

            elif req.HasField("action") and is_game_active:
                action_int = int(req.action.action)
                next_state_dict, i_player = env.step(action_int)
                is_over = env.is_over()

                await stream.send_message(
                    pb2.ProcessGameResponse(
                        state_update=pb2.GameStateUpdate(
                            state=json.dumps(next_state_dict, cls=NumpyEncoder),
                            is_over=is_over,
                            i_player=i_player,
                        )
                    )
                )
                if is_over:
                    payoffs = env.get_payoffs()
                    await stream.send_message(
                        pb2.ProcessGameResponse(
                            end_status=pb2.GameEndStatus(payoffs=payoffs.tolist())
                        )
                    )
                    is_game_active = False

            elif req.HasField("control"):
                control_type = req.control.type
                if control_type == pb2.GameControl.ControlType.PAUSE:
                    is_game_active = False
                elif control_type == pb2.GameControl.ControlType.RESUME and env is not None:
                    is_game_active = True
                    state_dict, i_player = env.reset()
                    await stream.send_message(
                        pb2.ProcessGameResponse(
                            state_update=pb2.GameStateUpdate(
                                state=json.dumps(state_dict, cls=NumpyEncoder),
                                is_over=False,
                                i_player=i_player,
                            )
                        )
                    )

        print("Client request stream closed.")
        # 当 async for 循环结束时，响应流会自动关闭

# --- grpclib 异步 Server 启动函数 ---
async def serve():
    # 1. 创建 grpclib Server 实例，并添加服务实例
    server = grpclib_server.Server([
        SponsorServiceServicer(), # <-- 依赖 __mapping__ 属性
    ])

    # 2. 绑定到地址和端口
    await server.start(HOST, PORT)
    
    print(f"gRPC Async Server (grpclib) started on {HOST}:{PORT}...")
    
    try:
        # 3. 阻塞，直到收到终止信号
        await server.wait_closed()
    except asyncio.CancelledError:
        print("Server shutdown initiated.")
    finally:
        # 4. 确保优雅关闭
        await server.close()

if __name__ == "__main__":
    try:
        asyncio.run(serve())
    except KeyboardInterrupt:
        # 允许通过 Ctrl+C 优雅退出
        pass
    except Exception as e:
        print(f"An unexpected error occurred: {e}")