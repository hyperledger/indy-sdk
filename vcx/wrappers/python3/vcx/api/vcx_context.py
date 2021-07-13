import asyncio
from contextlib import asynccontextmanager

from vcx.api.vcx_init import vcx_init, vcx_init_with_config
from vcx.common import shutdown

vcx_lock = asyncio.Lock()


@asynccontextmanager
async def vcx_context(config_path: str, delete_wallet: bool = False):
    await vcx_lock.acquire()
    await vcx_init(config_path)
    try:
        yield
    finally:
        shutdown(delete_wallet)
        vcx_lock.release()


@asynccontextmanager
async def vcx_context_with_config(config: str, delete_wallet: bool = False):
    await vcx_lock.acquire()
    await vcx_init_with_config(config)
    try:
        yield
    finally:
        shutdown(delete_wallet)
        vcx_lock.release()
