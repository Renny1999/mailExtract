import mailbox
from mailbox import mbox

if __name__=="__main__":
  fpath = "res/test.mbox"
  # file = open(fpath, 'rb')
  box = mbox(fpath)

  it = box.keys()

  m1 = box.get_message(it.pop())
  m1.set_flags("R")

  mfrom = m1.get_from()
  # print(mfrom)
  
  
