﻿using Indy.Sdk.Dotnet.Wrapper;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Indy.Sdk.Dotnet.Test.Wrapper.AgentTests
{
    public abstract class AgentIntegrationTestBase
    {
        protected static Wallet _wallet;
        protected static Pool _pool;
        protected string _poolName;
        private string _walletName = "agentWallet";

        [TestInitialize]
        public void SetUp()
        {
            InitHelper.Init();
            StorageUtils.CleanupStorage();

            _poolName = PoolUtils.CreatePoolLedgerConfig();

            var config2 = "{}";
            _pool = Pool.OpenPoolLedgerAsync(_poolName, config2).Result;

            Wallet.CreateWalletAsync(_poolName, _walletName, "default", null, null).Wait();
            _wallet = Wallet.OpenWalletAsync(_walletName, null, null).Result;
        }

        [TestCleanup]
        public void TearDown()
        {
            _wallet.CloseAsync().Wait();
            Wallet.DeleteWalletAsync(_walletName, null).Wait();

            _pool.CloseAsync().Wait();
            StorageUtils.CleanupStorage();
        }        
    }
}
