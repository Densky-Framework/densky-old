import { Socket, SocketCtx } from "densky";

export default function connect(_: SocketCtx, socket: Socket) {
  console.log("Socket is connected:", socket.id);
}
