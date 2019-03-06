=========
 vectool
=========

Vectool is a simple tool for finding nearest neighbours for vectors.
Its primary use is to make queries to a word vector model.

Building::

	cargo build --release

Building and installing::

	cargo install --path .

Usage
=====

Calculator mode
---------------

The calculator (calc) mode is used to find nearest neighbours of word vectors and their linear combinations.

In this example I use a `pre-trained fastText model <https://fasttext.cc/docs/en/english-vectors.html>`_.
The model is loaded using the following command::

	$ vectool crawl-300d-2M.vec calc
	Loaded 1999995 word model

Now we can, for example, find the nearest neighbours of the word 'king' (the first line is the input):

.. parsed-literal::

	*king*
	(king 1.0000)
	kings 0.7596
	queen 0.7075
	King 0.7049
	king. 0.6955
	king.The 0.6507
	monarch 0.6454
	prince 0.6444
	king- 0.6195
	kingly 0.6127

It is also possible make analogy queries by subtracting and adding words:

.. parsed-literal::

	*king - man + woman*
	(king 0.8531)
	queen 0.7660
	kings 0.6212
	queen-consort 0.6194
	monarch 0.6031
	Queen 0.5967
	King 0.5926
	king. 0.5914
	princess 0.5856
	queens 0.5800

The ``<>`` operator can be used to compare two vectors using both cosine and euclidean distance.

Filter mode
-----------

The filter mode is used to replace words in the input stream with their nearest neighbours.
One use case is to transform sentences to similar sentences.
This seems to work better for Finnish than for English.
The example uses `Yle's Finnish word2vec model <http://developer.yle.fi/data.html>`_
(convert from csv to tsv and remove the first line before using).

.. parsed-literal::

	**$ vectool word2vec_fin.tsv -e stopwords-fi.txt filter**
	Loaded 880327 word model
	*Kyselyt: Trumpin suosio alamaissa, demokraattiehdokkaat vahvoilla*
	Testitulokset: Obaman arvostus laskusuunnassa, demokraattiehdokkaat etulyöntiasemassa
	*Ylen toimittaja: Tilanne Venezuelan Kolumbian vastaisella rajalla erittäin kireä*
	Yleisradion ulkomaantoimittaja: Asetelma Zimbabwen Chilen levottomalla rajaalueella erittäin epävakaa

``-e`` is used to specify a file that contains a list of words to be excluded from the model.