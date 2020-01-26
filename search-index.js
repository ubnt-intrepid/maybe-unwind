var searchIndex={};
searchIndex["maybe_unwind"] = {"doc":"A variant of [`catch_unwind`] that also captures the panic…","i":[[3,"MaybeUnwind","maybe_unwind","A future for the [`maybe_unwind`] method.",null,null],[3,"CapturedPanicInfo","","An information about a panic.",null,null],[3,"UnwindContext","","The values captured due to a panic.",null,null],[12,"cause","","The cause of unwinding panic, caught by `catch_unwind`.",0,null],[12,"captured","","The panic information, caught in the panic hook.",0,null],[5,"set_hook","","Registers the custom panic hook so that the panic…",null,[[]]],[5,"reset_hook","","Unregisters the custom panic hook and reset the previous…",null,[[]]],[5,"maybe_unwind","","Invokes a closure, capturing the cause of an unwinding…",null,[[["f"]],[["result",["unwindcontext"]],["unwindcontext"]]]],[8,"FutureMaybeUnwindExt","","An extension trait for `Future`s that provides an adaptor…",null,null],[11,"maybe_unwind","","Catches unwinding panics while polling the future.",1,[[],["maybeunwind"]]],[11,"payload","","Return the payload associated with the panic.",2,[[["self"]],[["str"],["option",["str"]]]]],[11,"message","","Return the formatted panic message.",2,[[["self"]],["str"]]],[11,"file","","Return the name of the source file from which the panic…",2,[[["self"]],[["str"],["option",["str"]]]]],[11,"line","","Return the line number from which the panic originated.",2,[[["self"]],[["option",["u32"]],["u32"]]]],[11,"column","","Return the column from which the panic originated.",2,[[["self"]],[["option",["u32"]],["u32"]]]],[11,"backtrace","","Return the captured backtrace for the panic.",2,[[["self"]],["backtrace"]]],[11,"from","","",3,[[["t"]],["t"]]],[11,"into","","",3,[[],["u"]]],[11,"try_from","","",3,[[["u"]],["result"]]],[11,"try_into","","",3,[[],["result"]]],[11,"borrow","","",3,[[["self"]],["t"]]],[11,"borrow_mut","","",3,[[["self"]],["t"]]],[11,"type_id","","",3,[[["self"]],["typeid"]]],[11,"try_poll","","",3,[[["pin"],["f"],["context"]],["poll"]]],[11,"from","","",2,[[["t"]],["t"]]],[11,"into","","",2,[[],["u"]]],[11,"try_from","","",2,[[["u"]],["result"]]],[11,"try_into","","",2,[[],["result"]]],[11,"borrow","","",2,[[["self"]],["t"]]],[11,"borrow_mut","","",2,[[["self"]],["t"]]],[11,"type_id","","",2,[[["self"]],["typeid"]]],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"try_into","","",0,[[],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"type_id","","",0,[[["self"]],["typeid"]]],[11,"fmt","","",3,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",2,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",0,[[["formatter"],["self"]],["result"]]],[11,"poll","","",3,[[["self"],["pin"],["context"]],["poll"]]],[11,"maybe_unwind","","Catches unwinding panics while polling the future.",1,[[],["maybeunwind"]]]],"p":[[3,"UnwindContext"],[8,"FutureMaybeUnwindExt"],[3,"CapturedPanicInfo"],[3,"MaybeUnwind"]]};
addSearchOptions(searchIndex);initSearch(searchIndex);