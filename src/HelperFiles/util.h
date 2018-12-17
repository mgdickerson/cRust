#include <assert.h>
//  Assertions are used the same as assert!() in Rust,
//the author uses these to assert that certain aspects of
//what we program are true or as we believe them to be.

typedef char *string; //I think we can just use String::new(from()) for this.
string String(char *);

//typedef char bool;
#define TRUE 1
#define FALSE 0
//i believe true and false just work like this in Rust, no changes needed.

void *checked_malloc(int); //a malloc that will never return null (but does not guarentee space)

void *checked_malloc(int len) {
    void *p = malloc(len);
    assert(p);
    return p;
}
//  Not really sure what to do with this one. I think it can be handled with Box<> pointers
//and possibly other stack allocated members. I dont remember really needing to do malloc
//in the Rust tutorials, but I should check.

// #TODO: check Rust malloc //