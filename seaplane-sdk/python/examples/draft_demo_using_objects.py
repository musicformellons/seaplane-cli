from seaplane import Coprocessor, SmartPipe, coprocessor, log, start
from seaplane.logging import SeaLogger

log.level(SeaLogger.DEBUG)

smart_pipe = SmartPipe(
    func=(lambda input_data: "Toni " + str(input_data)),
    path="/inference_number",
    method="POST",
    id="my-smart-pipe",
)

convertInt = Coprocessor(func=(lambda data: int(data)), type="compute", id="convert-int")
doubleNumber = Coprocessor(func=(lambda data: data * 2), type="compute", id="multiply-by-two")

smart_pipe.add_coprocessor(convertInt)
smart_pipe.add_coprocessor(doubleNumber)

# start([smart_pipe]) Not working
