use crate::*;


use crate::simulator::*;
use crate::messages::*;
use crate::messages::InMsg::*;
use crate::messages::FileOp::*;



// Store env in a struct.
// - As opposed to using closures to close over the set of all used variables.
//      - This does not work as two sibling closures both cannot have read references to a parent scope.
pub struct EnvA {
    file: FilePath,
    i: I,
    o: O,
    r1: TxId,
    w1: TxId,
    c: InMsgId,
    w2: Option<TxId>,
}

impl EnvA {
    pub fn new(f: String) -> EnvA {
        let (i, o) = get_new_runtime_async();

        let a = i(File(GetReadTx(ArgsFile { file: f.clone() })));
        let b = i(File(GetWriteTx(ArgsFile { file: f.clone() })));
        let c = i(File(GetWriteTx(ArgsFile { file: f.clone() })));

        // Assert: `a` and `b` have responses.
        let mut h = get_response(&o, &vec![&a, &b]);

        // Assert: `c` is queued so there is no response.
        no_response(&o, &vec![&c]);


        match (h.remove(&a), h.remove(&b)) {
            (
                Some(OutMsg::File(Ok(TxIdOnly { tx_id: r1 }))),
                Some(OutMsg::File(Ok(TxIdOnly { tx_id: w1 })))) => {
                return EnvA {
                    file: f,
                    i,
                    o,
                    r1,
                    w1,
                    c,
                    w2: None,
                };
            }
            _ => unreachable!()
        }
    }

    pub fn w2_no_response_yet(&self) {
        let EnvA {
            o,
            c,
            ..
        } = self;

        no_response(&o, &vec![&c]);
    }

    pub fn w1_write_group_a(&self) {
        let EnvA {
            i,
            o,
            w1,
            ..
        } = self;


        let create = get_tx_q(&w1, "CREATE TABLE t1(a INTEGER PRIMARY KEY, b);");
        let insert = get_tx_q(&w1, "INSERT INTO t1 (b) VALUES (11), (22), (33)");
        let select = get_tx_q(&w1, "SELECT * FROM t1");


        let (op1, op2, op3) = (
            i(create),
            i(insert),
            i(select)
        );

        let reqs = vec![&op1, &op2, &op3];

        let h = o(
            &reqs,
            ms(20),
        ).unwrap();

        // dbg!(&h);

        match (
            h.get(&op1),
            h.get(&op2),
            h.get(&op3),
        ) {
            (
                Some(OutMsg::Tx(Ok(_))),
                Some(OutMsg::Tx(Ok(_))),
                Some(OutMsg::Tx(Ok(RSet { num_rows: 3, num_cols: 2, .. })))
            ) => {}
            _ => assert!(false)
        }
    }

    pub fn r1_cannot_see_group_a(&self) {
        let EnvA {
            i,
            o,
            r1,
            ..
        } = self;

        let select = i(get_tx_q(&r1, "SELECT * FROM t1"));
        let msg_ids = &vec![&select];
        let x = o(msg_ids, ms(20)).unwrap();


        match x.get(&select) {
            Some(OutMsg::Tx(Err(_))) => {
                // No table exists.
            }
            _ => assert!(false)
        }
    }

    pub fn w1_commit(&self) {
        let EnvA {
            i,
            o,
            w1,
            ..
        } = self;

        let commit = i(get_tx_commit(&w1));
        let msg_ids = &vec![&commit];
        let x = o(msg_ids, ms(20)).unwrap();
    }

    pub fn w1_rollback(&self) {
        let EnvA {
            i,
            o,
            w1,
            ..
        } = self;

        let commit = i(get_tx_rollback(&w1));
        let msg_ids = &vec![&commit];
        let x = o(msg_ids, ms(20)).unwrap();
    }

    pub fn w2_has_response(&mut self) {
        let EnvA {
            o,
            c,
            ..
        } = self;

        let mut h = get_response(&o, &vec![&c]);

        match h.remove(c) {
            Some(OutMsg::File(Ok(TxIdOnly { tx_id: w2 }))) => {
                self.w2 = Some(w2);
            }
            _ => unreachable!()
        }
    }

    pub fn w2_sees_group_a(&self) {
        match &self {
            EnvA { i, o, w2: Some(w2), .. } => {
                let select = i(get_tx_q(&w2, "SELECT * FROM t1"));
                let msg_ids = &vec![&select];
                let x = o(msg_ids, ms(20)).unwrap();


                match x.get(&select) {
                    Some(OutMsg::Tx(Ok(RSet { num_rows: 3, num_cols: 2, .. }))) => {}
                    _ => assert!(false)
                }
            }
            _ => unreachable!()
        }
    }

    pub fn w2_cannot_see_group_a(&self) {
        match &self {
            EnvA { i, o, w2: Some(w2), .. } => {
                let select = i(get_tx_q(&w2, "SELECT * FROM t1"));
                let msg_ids = &vec![&select];
                let x = o(msg_ids, ms(20)).unwrap();


                match x.get(&select) {
                    Some(OutMsg::Tx(Err(_))) => {
                        // No table exists.
                    }
                    _ => assert!(false)
                }
            }
            _ => unreachable!()
        }
    }

    pub fn r2_sees_group_a(&self) {
        let EnvA {
            i,
            o,
            file,
            ..
        } = self;

        let a = i(File(GetReadTx(ArgsFile { file: file.to_string() })));
        let mut h = get_response(&o, &vec![&a]);

        match h.remove(&a) {
            Some(OutMsg::File(Ok(TxIdOnly { tx_id: r2 }))) => {
                let select = i(get_tx_q(&r2, "SELECT * FROM t1"));
                let msg_ids = &vec![&select];
                let x = o(msg_ids, ms(20)).unwrap();

                match x.get(&select) {
                    Some(OutMsg::Tx(Ok(RSet { num_rows: 3, num_cols: 2, .. }))) => {}
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }
}