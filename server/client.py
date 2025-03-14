import socket
import json
import threading
import time
import struct
from dataclasses import dataclass
from typing import List, Optional, Dict, Any, Tuple

# Matching the server's data structures
@dataclass
class Position:
    x: float = 0.0
    y: float = 0.0
    z: float = 0.0
    
    def to_dict(self) -> Dict[str, float]:
        return {"x": self.x, "y": self.y, "z": self.z}

@dataclass
class Rotation:
    pitch: float = 0.0
    yaw: float = 0.0
    roll: float = 0.0
    
    def to_dict(self) -> Dict[str, float]:
        return {"pitch": self.pitch, "yaw": self.yaw, "roll": self.roll}

# Enum values to match Rust enum variants
class ClientMessageType:
    JOIN_GAME = "JoinGame"
    MOVE = "Move"
    SHOOT = "Shoot"

class ServerMessageType:
    GAME_START = "GameStart"
    PLAYERS_IN_LOBBY = "PlayersInLobby"
    PLAYER_MOVE = "PlayerMove"
    PLAYER_SHOOT = "PlayerShoot"
    PLAYER_DEATH = "PlayerDeath"
    PLAYER_SPAWN = "PlayerSpawn"
    HEALTH_UPDATE = "HealthUpdate"

class GameClient:
    def __init__(self, server_host="127.0.0.1", server_port=8080):
        self.server_addr = (server_host, server_port)
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.settimeout(0.1)  # Short timeout for non-blocking receive
        self.username = ""
        self.running = True
        
    def start(self):
        # Start the listener thread
        self.listener_thread = threading.Thread(target=self._listen_for_messages)
        self.listener_thread.daemon = True
        self.listener_thread.start()
        
        # Main menu loop
        self._show_main_menu()
    
    def stop(self):
        self.running = False
        self.listener_thread.join(timeout=1.0)
        self.sock.close()
        
    def _listen_for_messages(self):
        """Thread that listens for incoming server messages"""
        while self.running:
            try:
                data, addr = self.sock.recvfrom(1024)
                if data:
                    self._handle_server_message(data)
            except socket.timeout:
                # This is expected due to the timeout we set
                pass
            except Exception as e:
                print(f"Error receiving message: {e}")
                
    def _handle_server_message(self, data):
        try:
            # In a production system, we would use the same binary format as the server (bincode)
            # For this example, we'll use a simplified JSON-based approach
            # Assuming the server data structure is: [message_type, payload]
            message = json.loads(data.decode('utf-8'))
            msg_type = message.get("type")
            payload = message.get("data", {})
            
            print(f"\n--- RECEIVED {msg_type} ---")
            
            if msg_type == ServerMessageType.GAME_START:
                print("Game has started!")
                
            elif msg_type == ServerMessageType.PLAYERS_IN_LOBBY:
                print(f"Players in lobby: {payload['player_count']}")
                print(f"Player names: {', '.join(payload['players'])}")
                
            elif msg_type == ServerMessageType.PLAYER_MOVE:
                player_id = payload['player_id']
                pos = payload['position']
                rot = payload['rotation']
                print(f"Player {player_id} moved to pos:({pos['x']}, {pos['y']}, {pos['z']}), "
                      f"rot:({rot['pitch']}, {rot['yaw']}, {rot['roll']}), "
                      f"yield: {payload['yield_control']}")
                
            elif msg_type == ServerMessageType.PLAYER_SHOOT:
                player_id = payload['player_id']
                pos = payload['position']
                direction = payload['direction']
                print(f"Player {player_id} shot {payload['weapon_type']} from "
                      f"pos:({pos['x']}, {pos['y']}, {pos['z']}), "
                      f"direction:({direction['pitch']}, {direction['yaw']}, {direction['roll']})")
                
            elif msg_type == ServerMessageType.PLAYER_DEATH:
                killer_text = f" killed by {payload['killer_id']}" if payload.get('killer_id') else ""
                print(f"Player {payload['player_id']} died{killer_text}!")
                
            elif msg_type == ServerMessageType.PLAYER_SPAWN:
                player_id = payload['player_id']
                pos = payload['position']
                print(f"Player {player_id} spawned at pos:({pos['x']}, {pos['y']}, {pos['z']})")
                
            elif msg_type == ServerMessageType.HEALTH_UPDATE:
                print(f"Player {payload['player_id']} health updated to: {payload['health']}")
                
            else:
                print(f"Unknown message type: {msg_type}")
                print(f"Payload: {payload}")
                
            print("--- END OF MESSAGE ---\n")
            self._show_menu_prompt()
            
        except Exception as e:
            print(f"Error processing server message: {e}")
            
    def _show_main_menu(self):
        print("=== Game Client ===")
        self.username = input("Enter your username: ").strip()
        if not self.username:
            self.username = "Player"
            print(f"Using default username: {self.username}")
        
        # Send join game message
        self._send_join_game()
        
        # Show the menu
        while self.running:
            self._show_menu_prompt()
            try:
                choice = input().strip().lower()
                self._process_menu_choice(choice)
            except KeyboardInterrupt:
                print("\nExiting...")
                self.stop()
                break
                
    def _show_menu_prompt(self):
        print("\nSelect an action to perform:")
        print("1. Move")
        print("2. Shoot")
        print("q. Quit")
        print("> ", end='', flush=True)
                
    def _process_menu_choice(self, choice):
        if choice == '1':
            self._send_move()
        elif choice == '2':
            self._send_shoot()
        elif choice == 'q':
            print("Exiting...")
            self.stop()
            return False
        else:
            print("Invalid choice. Try again.")
        return True
                
    def _send_message(self, msg_type, payload):
        """Send a message to the server"""
        # In a production system, we would use the same binary format as the server
        # For this example, we'll use a simplified JSON-based approach
        message = {
            "type": msg_type,
            "data": payload
        }
        try:
            data = json.dumps(message).encode('utf-8')
            self.sock.sendto(data, self.server_addr)
            print(f"Sent {msg_type} message to server")
        except Exception as e:
            print(f"Error sending message: {e}")
                
    def _send_join_game(self):
        """Send a join game message to the server"""
        payload = {
            "username": self.username
        }
        self._send_message(ClientMessageType.JOIN_GAME, payload)
        
    def _send_move(self):
        """Send a move message with position and rotation"""
        try:
            print("\n=== Move Player ===")
            # Get position
            x = float(input("Position X: ") or "0")
            y = float(input("Position Y: ") or "0")
            z = float(input("Position Z: ") or "0")
            position = Position(x, y, z)
            
            # Get rotation
            pitch = float(input("Rotation Pitch: ") or "0")
            yaw = float(input("Rotation Yaw: ") or "0")
            roll = float(input("Rotation Roll: ") or "0")
            rotation = Rotation(pitch, yaw, roll)
            
            # Get yield control
            yield_control_input = input("Yield control (y/N): ").strip().lower()
            yield_control = yield_control_input == 'y'
            
            payload = {
                "position": position.to_dict(),
                "rotation": rotation.to_dict(),
                "yield_control": yield_control
            }
            
            self._send_message(ClientMessageType.MOVE, payload)
            
        except ValueError:
            print("Invalid input. Please enter numeric values.")
            
    def _send_shoot(self):
        """Send a shoot message with direction and weapon type"""
        try:
            print("\n=== Shoot ===")
            # Get direction
            pitch = float(input("Direction Pitch: ") or "0")
            yaw = float(input("Direction Yaw: ") or "0")
            roll = float(input("Direction Roll: ") or "0")
            direction = Rotation(pitch, yaw, roll)
            
            # Get weapon type
            weapon_type = input("Weapon Type (default: pistol): ").strip()
            if not weapon_type:
                weapon_type = "pistol"
                
            payload = {
                "direction": direction.to_dict(),
                "weapon_type": weapon_type
            }
            
            self._send_message(ClientMessageType.SHOOT, payload)
            
        except ValueError:
            print("Invalid input. Please enter numeric values.")

if __name__ == "__main__":
    print("Starting game client...")
    
    # Get server details
    server_host = input("Server host (default: localhost): ").strip()
    if not server_host:
        server_host = "localhost"
        
    try:
        server_port = int(input("Server port (default: 8080): ").strip())
    except ValueError:
        server_port = 8080
        
    print(f"Connecting to server at {server_host}:{server_port}")
    
    client = GameClient(server_host, server_port)
    try:
        client.start()
    except Exception as e:
        print(f"Error: {e}")
    finally:
        client.stop()
