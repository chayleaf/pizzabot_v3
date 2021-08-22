import os
import random
import time

from typing import Type, List, Dict, Tuple, Optional

import asyncio
import pineapplebot

from maubot import Plugin, MessageEvent
from maubot.handlers import command
from mautrix.types import EventType, MessageType, TextMessageEventContent
from mautrix.client.state_store.sqlalchemy import RoomState

SECRET = 'SOME_SECRET'
DATA_DIR = '/some/directory'

class Pizzabot(Plugin):
    pizzabot: pineapplebot.Pizzabot = pineapplebot.Pizzabot()
    loaded: bool = False

    async def start(self) -> None:
        await self.load_legacy()
        self.loaded = True

    async def stop(self) -> None:
        pass
    
    async def load_legacy(self) -> None:
        global DATA_DIR
        for fn in os.listdir(DATA_DIR): 
            if not fn.endswith('.txt'): continue
            path = os.path.join(DATA_DIR, fn)
            if fn[:-4] in ['661338676340719616', '606550060284510218']: continue
            self.pizzabot.load_file(fn[:-4], path)
            await asyncio.sleep(0)

    def get_response(self, text: str) -> Optional[str]:
        global SECRET
        return self.pizzabot.get_reply(text)
    
    def handle_msg(self, chid: str, content: str, is_self: bool) -> None:
        global SECRET
        chid = chid.replace('#', '').replace('<', '').replace('>', '').replace(':', '').replace('@', '').replace('!', '') # just in case
        with open(f'{DATA_DIR}/matrix_{chid}.txt', 'a', encoding='utf-8') as f:
            if is_self:
                print(SECRET + content, file=f)
            else:
                print(content, file=f)
        if is_self:
            self.pizzabot.set_message(f'matrix_{chid}', content)
        else:
            self.pizzabot.add_message(f'matrix_{chid}', content)

    @command.passive(r".*")
    async def msg_handler(self, event: MessageEvent, matches: List[Tuple[str, str]]) -> None:
        if not self.loaded:
            return
        if event.content.msgtype not in [MessageType.TEXT, MessageType.NOTICE]:
            return
        if not event.content.body:
            return
        is_self = event.sender == self.client.mxid
        self.handle_msg(f'{event.room_id}', event.content.body, is_self)
        if not RoomState.get(event.room_id).is_encrypted:
            return
        await event.mark_read()
        resp = self.get_response(event.content.body)
        if resp != None:
            await self.client.send_message_event(event.room_id, EventType.ROOM_MESSAGE, TextMessageEventContent(msgtype=MessageType.TEXT, body=resp))

