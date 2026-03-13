                     ┌─────────────┐ 
                     │     VPS     │                        
                     │             │
     ┌───────────────┤  STUN srv   │◄──────────────┐
     │  "what's my   │  (UDP 3478) │   "what's my  │
     │   ext addr?"  ├─────────────┤    ext addr?" │
     │               │             │               │
     │  ┌────────────┤  Signal srv │◄───────────┐  │
     │  │ WebSocket  │  (TCP/WS)   │  WebSocket │  │
     │  │            └─────────────┘            │  │
     │  │                                       │  │
     ▼  ▼                                       ▼  ▼
┌──────────┐                                 ┌──────────┐
│  Peer A  │─── direct UDP (encrypted) ────▶ │  Peer B  │
│  behind  │◄── direct UDP (encrypted) ───── │  behind  │
│  NAT     │                                 │  NAT     │
└──────────┘                                 └──────────┘
