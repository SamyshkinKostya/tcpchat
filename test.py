from random import randint

while True:
    text = ""

    for i in range(50):
        line = randint(0, 24)
        col = randint(0, 80)
        fr = randint(0, 255)
        fg = randint(0, 255)
        fb = randint(0, 255)
        br = randint(0, 255)
        bg = randint(0, 255)
        bb = randint(0, 255)
        char = chr(randint(0, 126))

        # \x1B[6n
        text += f"\x1B[6n\x1B[38;2;{fr};{fg};{fb}m\x1B[48;2;{br};{bg};{bb}m\x1B[{line};{col}H\a{char}"

    print(f"{text}")