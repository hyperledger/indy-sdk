import asyncio

import pytest

from vcx.api.vcx_init import vcx_init_with_config
from vcx.api.vcx_context import vcx_context_with_config


@pytest.fixture
def set_lock_event_loop(event_loop):
    from vcx.api.vcx_context import vcx_lock

    # pytest uses its own loop, so this ugliness is necessary for the test to not explode.
    vcx_lock._loop = event_loop


@pytest.mark.asyncio
async def test_vcx_init(set_lock_event_loop):
    from random import random

    async def run():
        async with vcx_context_with_config("ENABLE_TEST_MODE"):
            await asyncio.sleep(random() / 10000)

    await asyncio.gather(*(run() for _ in range(10)))
