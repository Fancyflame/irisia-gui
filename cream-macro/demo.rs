struct MyStruct {
    // Get and setby default
    #[attr]
    data1: String,
    // Read-only
    #[attr(get)]
    data2: String,
    // Set-only
    #[attr(set)]
    data3: String,
    // Get and set with specified function
    #[attr(get=get_fn1, set=set_fn1)]
    data4: String,
    // Set with specified function and get by default
    #[attr(get, set=set_fn2)]
    data5: String,
    // Not an attribute
    data6: String,
}

// Generates

impl MyStruct {
    #[inline]
    fn __ChampagneGUIAutoGen__data1_get(&self) -> &String {
        &self.data1
    }

    #[inline]
    fn __ChampagneGUIAutoGen__data1_set(&mut self, data: String) {
        self.data1 = data;
    }

    // ...

    #[inline]
    fn __ChampagneGUIAutoGen__data5_set(&mut self, data: String) {
        let old = ::std::mem::replace(&mut self.data5, data);
        self.set_fn2(old); // mut
    }
}
