import string

# naive solver
def search(edges, current_edge, current_word, remaining_len):
    # print(f">search {current_edge}, {current_word}, {remaining_len}")
    if remaining_len == 0:
        return set([current_word])
    res = set()
    for next_edge in range(4):
        if next_edge == current_edge:
            continue
        for next_letter in range(3):
            l = edges[next_edge][next_letter]
            res.update(search(edges, next_edge, current_word + l, remaining_len - 1))
    return res

def solve_letter_boxed(edges):
    all_words = set()
    for starting_edge in range(4):
        for starting_letter in range(3):
            w = edges[starting_edge][starting_letter]
            for word_len in range(1,8):
                all_words.update(search(edges, starting_edge, w, word_len))

    valid_words = all_words.intersection(filtered)
    print(f"Number of valid words: {len(valid_words)}")
    # greedy solution: try the largest entropy words first
    best_first = sorted(valid_words, key=lambda s: entropy(s), reverse=True)
    solutions = []
    for good_start in best_first:
        chain = [good_start]
        while not covers_all(chain):
            could_be_next = filter(lambda next: chain[-1][-1] == next[0], valid_words)
            best = sorted(could_be_next, key=lambda w: cover(chain + [w]), reverse=True)[0]
            chain.append(best)

        if len(chain) == 2:
            print(f"Found a two word solution: {chain}")

        if len(chain) <= 3:
            solutions.append(chain)
    return solutions

def covers_all(chain):
    # print(chain)
    return cover(chain) == 12

def cover(chain):
    return len(set([letter for word in chain for letter in word]))

def entropy(word):
    return len(set(word))

def get_words(filename):
    return [s.strip().lower() for s in open(filename).readlines()]

def filter_wordlist(words):
    return list(filter(lambda s: all([c in string.ascii_letters for c in s]), words))

wordlist = get_words("/usr/share/dict/words")
filtered = set(filter_wordlist(wordlist))

# edges = [["r", "s", "h"], ["w", "k", "b"], ["d", "e", "l"], ["y", "i", "a"]]
edges = [["n", "k", "c"], ["d", "t", "v"], ["r", "m", "o"], ["a", "i", "w"]]
solutions = solve_letter_boxed(edges)
for solution in solutions:
    print(f"Solution = {solution}, len = {len(solution)}")

# solver v2
# build a map of prefix -> all possible letters that could follow that prefix and form a valid word
# if the prefix map contains all prefixes of length < 5 then we could probably then bruteforce the rest
# and generate a list of all possible words creatable by the given letterbox
# build this prefixmap in O(n) time somehow (n = length of dictionary)