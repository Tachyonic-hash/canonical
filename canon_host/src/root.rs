use canonical::{Canon, Store};

use crate::{Apply, Execute, Query, Transaction};

/// A Store that can perist state.
pub trait Persistent: Store {
    fn set_root(&mut self, root: Self::Ident);
    fn get_root(&self) -> Option<Self::Ident>;
}

/// The root of the whole-network state, including
pub struct Root<State, S>
where
    S: Store,
{
    #[allow(unused)]
    state: State,
    #[allow(unused)]
    store: S,
}

impl<State, S> Root<State, S>
where
    State: Default + Canon<S>,
    S: Persistent,
{
    /// Creates a new root from a persistent store
    pub fn new(store: S) -> Result<Self, S::Error> {
        let state = match store.get_root() {
            Some(ref root) => store.get(root)?,
            None => Default::default(),
        };
        Ok(Root { state, store })
    }
}

impl<A, R, State, S, const ID: u8> Apply<A, R, S, ID> for Root<State, S>
where
    State: Apply<A, R, S, ID>,
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<A, R, ID>,
    ) -> Result<R, S::Error> {
        self.state.apply(transaction)
    }
}

impl<A, R, State, S, const ID: u8> Execute<A, R, S, ID> for Root<State, S>
where
    State: Execute<A, R, S, ID>,
    S: Store,
{
    fn execute(&self, query: Query<A, R, ID>) -> Result<R, S::Error> {
        self.state.execute(query)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{DiskStore, Remote, Transaction, Wasm};

    use canonical::{Canon, Store};
    use canonical_derive::Canon;

    use counter::{Counter, Query, READ_VALUE};

    type ContractId = usize;

    const ADD_CONTRACT: u8 = 0;
    const APPLY_CONTRACT_TRANSACTION: u8 = 1;
    const EXECUTE_CONTRACT_QUERY: u8 = 2;

    #[derive(Clone, Canon, Default)]
    struct TestState<S: Store> {
        opcount: u64,
        contracts: Vec<Remote<S>>,
    }

    impl<S> TestState<S>
    where
        S: Store,
    {
        fn add_contract(
            contract: Remote<S>,
        ) -> Transaction<Remote<S>, ContractId, ADD_CONTRACT> {
            Transaction::new(contract)
        }

        fn execute_contract_query(
            contract: Remote<S>,
        ) -> Transaction<Remote<S>, ContractId, ADD_CONTRACT> {
            Transaction::new(contract)
        }
    }

    impl<S> Apply<Remote<S>, usize, S, ADD_CONTRACT> for TestState<S>
    where
        S: Store,
    {
        fn apply(
            &mut self,
            transaction: Transaction<Remote<S>, usize, ADD_CONTRACT>,
        ) -> Result<usize, S::Error> {
            let id = self.contracts.len();
            self.contracts.push(transaction.into_args());
            Ok(id)
        }
    }

    impl<A, R, S, const ID: u8>
        Execute<Query<A, R, ID>, R, S, EXECUTE_CONTRACT_QUERY> for TestState<S>
    where
        S: Store,
    {
        fn execute(
            &self,
            query: Query<Query<A, R, ID>, R, EXECUTE_CONTRACT_QUERY>,
        ) -> Result<R, S::Error> {
            todo!()
        }
    }

    #[test]
    fn create_root() {
        let dir = tempfile::tempdir().unwrap();
        let store = DiskStore::new(dir.path()).unwrap();
        let mut state: Root<TestState<DiskStore>, _> =
            Root::new(store.clone()).unwrap();

        let counter = Counter::new(13);

        let wasm_counter = Wasm::new(
            // unlucky number to not get too lucky in testing
            Counter::new(13),
            include_bytes!(
                "../../module_examples/modules/counter/counter.wasm"
            ),
        );

        // hide counter behind a remote to erase the type
        let remote = Remote::new(wasm_counter, store.clone()).unwrap();

        let transaction = TestState::<DiskStore>::add_contract(remote);
        let id = state.apply(transaction).unwrap();

        let counter_query: Query<(), i32, READ_VALUE> = Counter::read_value();

        // let wrapped_query = crate::Query::<
        //     crate::Query<(), i32, READ_VALUE>,
        //     i32,
        //     EXECUTE_CONTRACT_QUERY,
        // >::new(counter_query);

        // let id = state.execute(wrapped_query).unwrap();

        // assert_eq!(id, 0);
    }
}
