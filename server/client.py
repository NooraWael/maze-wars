import asyncio
import socket
import json
from enum import Enum
from dataclasses import dataclass
from typing import Optional

class EventType(Enum):
    JOIN_GAME = "JoinGame"
    MOVE = "Move"
    SHOOT = "Shoot"

@dataclass
class Position:
    x: float
    y: float
    z: float

@dataclass
class Rotation:
    pitch: float
    yaw: float
    roll: float

@dataclass
class ClientMessage:
    event_type: EventType
    username: Optional[str] = None
    position: Optional[Position] = None
    rotation: Optional[Rotation] = None
    weapon_type: Optional[str] = None

class GameClient:
    def __init__(self, server_ip: str, port: int):
        self.server_addr = (server_ip, port)
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setblocking(False)
        self.joined = False

    async def send_message(self, message: ClientMessage):
        try:
            data = json.dumps({
                "type": message.event_type.value,
                "data": {
                    "username": message.username,
                    "position": message.position.__dict__ if message.position else None,
                    "rotation": message.rotation.__dict__ if message.rotation else None,
                    "weapon_type": message.weapon_type
                }
            }).encode('utf-8')
            if message.event_type != EventType.JOIN_GAME and not self.joined:
                raise ConnectionError("Must join game before sending other events")
            
            await asyncio.wait_for(
                asyncio.get_event_loop().sock_sendto(self.sock, data, self.server_addr),
                timeout=1.0
            )
            if message.event_type == EventType.JOIN_GAME:
                self.joined = True
        except asyncio.TimeoutError:
            print("\nError: Server not responding - check if server is running")
            raise
        except ConnectionError as e:
            print(f"\nError: {e}")
            raise
        except Exception as e:
            print(f"\nError sending message: {e}")
            raise

    async def receive_messages(self):
        while True:
            try:
                data, _ = await asyncio.get_event_loop().sock_recvfrom(self.sock, 1024)
                print(f"\nReceived server update: {data.decode()}")
            except BlockingIOError:
                await asyncio.sleep(0.1)
            except Exception as e:
                print(f"\nError receiving message: {e}")
                break

async def user_input_handler(client: GameClient):
    while True:
        try:
            print("\nSelect event to send:")
            print("1. Join Game")
            print("2. Move")
            print("3. Shoot")
            choice = input("Enter choice (1-3): ")

            if choice == "1":
                username = input("Enter username: ")
                msg = ClientMessage(EventType.JOIN_GAME, username=username)
            elif choice == "2":
                x = float(input("X position: "))
                y = float(input("Y position: "))
                z = float(input("Z position: "))
                pitch = float(input("Pitch: "))
                yaw = float(input("Yaw: "))
                roll = float(input("Roll: "))
                msg = ClientMessage(EventType.MOVE,
                    position=Position(x, y, z),
                    rotation=Rotation(pitch, yaw, roll))
            elif choice == "3":
                weapon = input("Weapon type: ")
                msg = ClientMessage(EventType.SHOOT, weapon_type=weapon)
            else:
                print("Invalid choice")
                continue

            try:
                await client.send_message(msg)
                print("Event sent successfully")
            except Exception as e:
                print(f"Failed to send event: {e}")

        except ValueError as e:
            print(f"Invalid input: {e}")
        except Exception as e:
            print(f"Error: {e}")

async def main():
    client = GameClient("0.0.0.0", 2025)
    await asyncio.gather(
        user_input_handler(client),
        client.receive_messages()
    )

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nClient shutdown")