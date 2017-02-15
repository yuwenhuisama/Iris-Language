/* A Bison parser, made by GNU Bison 2.7.  */

/* Bison implementation for Yacc-like parsers in C
   
      Copyright (C) 1984, 1989-1990, 2000-2012 Free Software Foundation, Inc.
   
   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.
   
   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.
   
   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.
   
   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* C LALR(1) parser skeleton written by Richard Stallman, by
   simplifying the original so-called "semantic" parser.  */

/* All symbols defined below should begin with yy or YY, to avoid
   infringing on user name space.  This should be done even for local
   variables, as they might otherwise be expanded by user macros.
   There are some unavoidable exceptions within include files to
   define necessary library symbols; they are noted "INFRINGES ON
   USER NAME SPACE" below.  */

/* Identify Bison output.  */
#define YYBISON 1

/* Bison version.  */
#define YYBISON_VERSION "2.7"

/* Skeleton name.  */
#define YYSKELETON_NAME "yacc.c"

/* Pure parsers.  */
#define YYPURE 0

/* Push parsers.  */
#define YYPUSH 0

/* Pull parsers.  */
#define YYPULL 1




/* Copy the first part of user declarations.  */
/* Line 371 of yacc.c  */
#line 1 "iris.y"

#include "IrisCompilerDev.h"
#include <sstream>
using namespace std;
//#define YYDEBUG 1

extern int yylex();

int yyerror(char const *str)
{
	extern char *yytext;

	string strMessage = "<Irregular : SyntaxIrregular>\n  Irregular-happened Report : Oh! Master, an Irregular has happened and Iris is not clever and dosen't kown how to deal with it. Can you please cheak it yourself? \n";
	string strIrregularMessage = ">The Irregular name is ";
	stringstream ssStream;
	ssStream << IrisCompiler::CurrentCompiler()->GetCurrentLineNumber();
	string strLinenoMessage = ">and happened at line " + ssStream.str() + " where token is " + string(yytext) + ".";

	strMessage += strIrregularMessage + "SyntaxIrregular," + "\n" + strLinenoMessage + "\n";

	IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorDirectly(strMessage);
	return 0;
}

/* Line 371 of yacc.c  */
#line 93 "y.tab.c"

# ifndef YY_NULL
#  if defined __cplusplus && 201103L <= __cplusplus
#   define YY_NULL nullptr
#  else
#   define YY_NULL 0
#  endif
# endif

/* Enabling verbose error messages.  */
#ifdef YYERROR_VERBOSE
# undef YYERROR_VERBOSE
# define YYERROR_VERBOSE 1
#else
# define YYERROR_VERBOSE 0
#endif

/* In a future release of Bison, this section will be replaced
   by #include "y.tab.h".  */
#ifndef YY_YY_Y_TAB_H_INCLUDED
# define YY_YY_Y_TAB_H_INCLUDED
/* Enabling traces.  */
#ifndef YYDEBUG
# define YYDEBUG 0
#endif
#if YYDEBUG
extern int yydebug;
#endif

/* Tokens.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
   /* Put the tokens into the symbol table, so that GDB and other debuggers
      know about them.  */
   enum yytokentype {
     INTEGER = 258,
     FLOAT = 259,
     STRING = 260,
     IDENTIFIER = 261,
     FUNCTION = 262,
     CLASS = 263,
     MODULE = 264,
     INTERFACE = 265,
     EXTENDS = 266,
     INVOLVES = 267,
     JOINTS = 268,
     PERSONAL = 269,
     RELATIVE = 270,
     EVERYONE = 271,
     GET = 272,
     SET = 273,
     GSET = 274,
     IF = 275,
     ELSE = 276,
     ELSEIF = 277,
     FOR = 278,
     IN = 279,
     SWITCH = 280,
     WHEN = 281,
     CAST = 282,
     ITERATOR = 283,
     SUPER = 284,
     BREAK = 285,
     CONTINUE = 286,
     RETURN = 287,
     BLOCK = 288,
     SELF = 289,
     WITH = 290,
     WITHOUT = 291,
     ORDER = 292,
     SERVE = 293,
     IGNORE = 294,
     GROAN = 295,
     TRUE = 296,
     FALSE = 297,
     NIL = 298,
     DOT = 299,
     ALIAS = 300,
     RETRY = 301,
     REDO = 302,
     GOTO = 303,
     CONST = 304,
     GLOBAL = 305,
     STATIC = 306,
     LP = 307,
     RP = 308,
     LC = 309,
     RC = 310,
     LB = 311,
     RB = 312,
     SEMICOLON = 313,
     COMMA = 314,
     LOGICAL_AND = 315,
     LOGICAL_OR = 316,
     LOGICAL_NOT = 317,
     ASSIGN = 318,
     ADD = 319,
     SUB = 320,
     MUL = 321,
     DIV = 322,
     MOD = 323,
     POWER = 324,
     BIT_AND = 325,
     BIT_OR = 326,
     BIT_XOR = 327,
     BIT_NOT = 328,
     ASSIGN_ADD = 329,
     ASSIGN_SUB = 330,
     ASSIGN_MUL = 331,
     ASSIGN_DIV = 332,
     ASSIGN_MOD = 333,
     ASSIGN_POWER = 334,
     ASSIGN_BIT_AND = 335,
     ASSIGN_BIT_OR = 336,
     ASSIGN_BIT_XOR = 337,
     BIT_SHR = 338,
     EQ = 339,
     NE = 340,
     GT = 341,
     GE = 342,
     LT = 343,
     LE = 344,
     BIT_SHL = 345,
     BIT_SAR = 346,
     BIT_SAL = 347,
     ASSIGN_BIT_SHR = 348,
     ASSIGN_BIT_SHL = 349,
     ASSIGN_BIT_SAR = 350,
     ASSIGN_BIT_SAL = 351,
     ITER = 352,
     FILED = 353,
     SHARP = 354,
     CONLON = 355
   };
#endif
/* Tokens.  */
#define INTEGER 258
#define FLOAT 259
#define STRING 260
#define IDENTIFIER 261
#define FUNCTION 262
#define CLASS 263
#define MODULE 264
#define INTERFACE 265
#define EXTENDS 266
#define INVOLVES 267
#define JOINTS 268
#define PERSONAL 269
#define RELATIVE 270
#define EVERYONE 271
#define GET 272
#define SET 273
#define GSET 274
#define IF 275
#define ELSE 276
#define ELSEIF 277
#define FOR 278
#define IN 279
#define SWITCH 280
#define WHEN 281
#define CAST 282
#define ITERATOR 283
#define SUPER 284
#define BREAK 285
#define CONTINUE 286
#define RETURN 287
#define BLOCK 288
#define SELF 289
#define WITH 290
#define WITHOUT 291
#define ORDER 292
#define SERVE 293
#define IGNORE 294
#define GROAN 295
#define TRUE 296
#define FALSE 297
#define NIL 298
#define DOT 299
#define ALIAS 300
#define RETRY 301
#define REDO 302
#define GOTO 303
#define CONST 304
#define GLOBAL 305
#define STATIC 306
#define LP 307
#define RP 308
#define LC 309
#define RC 310
#define LB 311
#define RB 312
#define SEMICOLON 313
#define COMMA 314
#define LOGICAL_AND 315
#define LOGICAL_OR 316
#define LOGICAL_NOT 317
#define ASSIGN 318
#define ADD 319
#define SUB 320
#define MUL 321
#define DIV 322
#define MOD 323
#define POWER 324
#define BIT_AND 325
#define BIT_OR 326
#define BIT_XOR 327
#define BIT_NOT 328
#define ASSIGN_ADD 329
#define ASSIGN_SUB 330
#define ASSIGN_MUL 331
#define ASSIGN_DIV 332
#define ASSIGN_MOD 333
#define ASSIGN_POWER 334
#define ASSIGN_BIT_AND 335
#define ASSIGN_BIT_OR 336
#define ASSIGN_BIT_XOR 337
#define BIT_SHR 338
#define EQ 339
#define NE 340
#define GT 341
#define GE 342
#define LT 343
#define LE 344
#define BIT_SHL 345
#define BIT_SAR 346
#define BIT_SAL 347
#define ASSIGN_BIT_SHR 348
#define ASSIGN_BIT_SHL 349
#define ASSIGN_BIT_SAR 350
#define ASSIGN_BIT_SAL 351
#define ITER 352
#define FILED 353
#define SHARP 354
#define CONLON 355



#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
typedef union YYSTYPE
{
/* Line 387 of yacc.c  */
#line 26 "iris.y"

	IrisIdentifier* m_pIdentifier = nullptr;
	IrisList<IrisIdentifier*>* m_pIdentifierList;
	IrisExpression* m_pExpression; 
	IrisList<IrisExpression*>* m_pExpressionList;
	IrisStatement* m_pStatement;
	IrisList<IrisStatement*>* m_pStatementList;
	IrisBinaryExpressionType m_eExpressionType;
	IrisBlock* m_pBlock;
	IrisFunctionHeader* m_pFunctionHeader;
	IrisConditionIfStatement* m_pConditionIf;
	IrisLoopIfStatement* m_pLoopIf;
	IrisElseIf* m_pElseIf;
	IrisList<IrisElseIf*>* m_pElseIfList;
	IrisWhen* m_pWhen;
	IrisList<IrisWhen*>* m_pWhenList;
	IrisSwitchBlock* m_pSwitchBlock;
	IrisHashPair* m_pHashPair;
	IrisList<IrisHashPair*>* m_pHashPairList;
	IrisClosureBlockLiteral* m_pClosureBlockLiteral;


/* Line 387 of yacc.c  */
#line 359 "y.tab.c"
} YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define yystype YYSTYPE /* obsolescent; will be withdrawn */
# define YYSTYPE_IS_DECLARED 1
#endif

extern YYSTYPE yylval;

#ifdef YYPARSE_PARAM
#if defined __STDC__ || defined __cplusplus
int yyparse (void *YYPARSE_PARAM);
#else
int yyparse ();
#endif
#else /* ! YYPARSE_PARAM */
#if defined __STDC__ || defined __cplusplus
int yyparse (void);
#else
int yyparse ();
#endif
#endif /* ! YYPARSE_PARAM */

#endif /* !YY_YY_Y_TAB_H_INCLUDED  */

/* Copy the second part of user declarations.  */

/* Line 390 of yacc.c  */
#line 387 "y.tab.c"

#ifdef short
# undef short
#endif

#ifdef YYTYPE_UINT8
typedef YYTYPE_UINT8 yytype_uint8;
#else
typedef unsigned char yytype_uint8;
#endif

#ifdef YYTYPE_INT8
typedef YYTYPE_INT8 yytype_int8;
#elif (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
typedef signed char yytype_int8;
#else
typedef short int yytype_int8;
#endif

#ifdef YYTYPE_UINT16
typedef YYTYPE_UINT16 yytype_uint16;
#else
typedef unsigned short int yytype_uint16;
#endif

#ifdef YYTYPE_INT16
typedef YYTYPE_INT16 yytype_int16;
#else
typedef short int yytype_int16;
#endif

#ifndef YYSIZE_T
# ifdef __SIZE_TYPE__
#  define YYSIZE_T __SIZE_TYPE__
# elif defined size_t
#  define YYSIZE_T size_t
# elif ! defined YYSIZE_T && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#  include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  define YYSIZE_T size_t
# else
#  define YYSIZE_T unsigned int
# endif
#endif

#define YYSIZE_MAXIMUM ((YYSIZE_T) -1)

#ifndef YY_
# if defined YYENABLE_NLS && YYENABLE_NLS
#  if ENABLE_NLS
#   include <libintl.h> /* INFRINGES ON USER NAME SPACE */
#   define YY_(Msgid) dgettext ("bison-runtime", Msgid)
#  endif
# endif
# ifndef YY_
#  define YY_(Msgid) Msgid
# endif
#endif

/* Suppress unused-variable warnings by "using" E.  */
#if ! defined lint || defined __GNUC__
# define YYUSE(E) ((void) (E))
#else
# define YYUSE(E) /* empty */
#endif

/* Identity function, used to suppress warnings about constant conditions.  */
#ifndef lint
# define YYID(N) (N)
#else
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static int
YYID (int yyi)
#else
static int
YYID (yyi)
    int yyi;
#endif
{
  return yyi;
}
#endif

#if ! defined yyoverflow || YYERROR_VERBOSE

/* The parser invokes alloca or malloc; define the necessary symbols.  */

# ifdef YYSTACK_USE_ALLOCA
#  if YYSTACK_USE_ALLOCA
#   ifdef __GNUC__
#    define YYSTACK_ALLOC __builtin_alloca
#   elif defined __BUILTIN_VA_ARG_INCR
#    include <alloca.h> /* INFRINGES ON USER NAME SPACE */
#   elif defined _AIX
#    define YYSTACK_ALLOC __alloca
#   elif defined _MSC_VER
#    include <malloc.h> /* INFRINGES ON USER NAME SPACE */
#    define alloca _alloca
#   else
#    define YYSTACK_ALLOC alloca
#    if ! defined _ALLOCA_H && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#     include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
      /* Use EXIT_SUCCESS as a witness for stdlib.h.  */
#     ifndef EXIT_SUCCESS
#      define EXIT_SUCCESS 0
#     endif
#    endif
#   endif
#  endif
# endif

# ifdef YYSTACK_ALLOC
   /* Pacify GCC's `empty if-body' warning.  */
#  define YYSTACK_FREE(Ptr) do { /* empty */; } while (YYID (0))
#  ifndef YYSTACK_ALLOC_MAXIMUM
    /* The OS might guarantee only one guard page at the bottom of the stack,
       and a page size can be as small as 4096 bytes.  So we cannot safely
       invoke alloca (N) if N exceeds 4096.  Use a slightly smaller number
       to allow for a few compiler-allocated temporary stack slots.  */
#   define YYSTACK_ALLOC_MAXIMUM 4032 /* reasonable circa 2006 */
#  endif
# else
#  define YYSTACK_ALLOC YYMALLOC
#  define YYSTACK_FREE YYFREE
#  ifndef YYSTACK_ALLOC_MAXIMUM
#   define YYSTACK_ALLOC_MAXIMUM YYSIZE_MAXIMUM
#  endif
#  if (defined __cplusplus && ! defined EXIT_SUCCESS \
       && ! ((defined YYMALLOC || defined malloc) \
	     && (defined YYFREE || defined free)))
#   include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#   ifndef EXIT_SUCCESS
#    define EXIT_SUCCESS 0
#   endif
#  endif
#  ifndef YYMALLOC
#   define YYMALLOC malloc
#   if ! defined malloc && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void *malloc (YYSIZE_T); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
#  ifndef YYFREE
#   define YYFREE free
#   if ! defined free && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void free (void *); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
# endif
#endif /* ! defined yyoverflow || YYERROR_VERBOSE */


#if (! defined yyoverflow \
     && (! defined __cplusplus \
	 || (defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yytype_int16 yyss_alloc;
  YYSTYPE yyvs_alloc;
};

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (sizeof (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (sizeof (yytype_int16) + sizeof (YYSTYPE)) \
      + YYSTACK_GAP_MAXIMUM)

# define YYCOPY_NEEDED 1

/* Relocate STACK from its old location to the new one.  The
   local variables YYSIZE and YYSTACKSIZE give the old and new number of
   elements in the stack, and YYPTR gives the new location of the
   stack.  Advance YYPTR to a properly aligned location for the next
   stack.  */
# define YYSTACK_RELOCATE(Stack_alloc, Stack)				\
    do									\
      {									\
	YYSIZE_T yynewbytes;						\
	YYCOPY (&yyptr->Stack_alloc, Stack, yysize);			\
	Stack = &yyptr->Stack_alloc;					\
	yynewbytes = yystacksize * sizeof (*Stack) + YYSTACK_GAP_MAXIMUM; \
	yyptr += yynewbytes / sizeof (*yyptr);				\
      }									\
    while (YYID (0))

#endif

#if defined YYCOPY_NEEDED && YYCOPY_NEEDED
/* Copy COUNT objects from SRC to DST.  The source and destination do
   not overlap.  */
# ifndef YYCOPY
#  if defined __GNUC__ && 1 < __GNUC__
#   define YYCOPY(Dst, Src, Count) \
      __builtin_memcpy (Dst, Src, (Count) * sizeof (*(Src)))
#  else
#   define YYCOPY(Dst, Src, Count)              \
      do                                        \
        {                                       \
          YYSIZE_T yyi;                         \
          for (yyi = 0; yyi < (Count); yyi++)   \
            (Dst)[yyi] = (Src)[yyi];            \
        }                                       \
      while (YYID (0))
#  endif
# endif
#endif /* !YYCOPY_NEEDED */

/* YYFINAL -- State number of the termination state.  */
#define YYFINAL  128
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   868

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  101
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  70
/* YYNRULES -- Number of rules.  */
#define YYNRULES  232
/* YYNRULES -- Number of states.  */
#define YYNSTATES  460

/* YYTRANSLATE(YYLEX) -- Bison symbol number corresponding to YYLEX.  */
#define YYUNDEFTOK  2
#define YYMAXUTOK   355

#define YYTRANSLATE(YYX)						\
  ((unsigned int) (YYX) <= YYMAXUTOK ? yytranslate[YYX] : YYUNDEFTOK)

/* YYTRANSLATE[YYLEX] -- Bison symbol number corresponding to YYLEX.  */
static const yytype_uint8 yytranslate[] =
{
       0,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     1,     2,     3,     4,
       5,     6,     7,     8,     9,    10,    11,    12,    13,    14,
      15,    16,    17,    18,    19,    20,    21,    22,    23,    24,
      25,    26,    27,    28,    29,    30,    31,    32,    33,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,    53,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    66,    67,    68,    69,    70,    71,    72,    73,    74,
      75,    76,    77,    78,    79,    80,    81,    82,    83,    84,
      85,    86,    87,    88,    89,    90,    91,    92,    93,    94,
      95,    96,    97,    98,    99,   100
};

#if YYDEBUG
/* YYPRHS[YYN] -- Index of the first RHS symbol of rule number YYN in
   YYRHS.  */
static const yytype_uint16 yyprhs[] =
{
       0,     0,     3,     4,     6,     8,    11,    14,    16,    18,
      20,    22,    24,    26,    28,    30,    32,    34,    36,    38,
      40,    42,    44,    46,    48,    50,    52,    54,    56,    58,
      65,    66,    69,    72,    73,    76,    77,    80,    82,    84,
      88,    92,    98,   106,   112,   121,   127,   133,   141,   147,
     155,   161,   169,   174,   179,   182,   189,   194,   200,   209,
     216,   223,   231,   242,   251,   254,   259,   265,   268,   272,
     274,   276,   278,   280,   282,   284,   286,   288,   290,   292,
     294,   296,   298,   300,   302,   304,   306,   308,   310,   312,
     314,   316,   318,   320,   322,   328,   336,   343,   352,   354,
     357,   363,   372,   373,   376,   382,   386,   392,   394,   397,
     403,   411,   423,   432,   433,   438,   441,   445,   448,   451,
     457,   464,   474,   482,   488,   491,   496,   502,   505,   509,
     511,   514,   516,   520,   522,   524,   526,   528,   530,   532,
     534,   536,   538,   540,   542,   544,   546,   548,   550,   554,
     556,   560,   562,   566,   568,   572,   574,   578,   580,   584,
     588,   590,   594,   598,   602,   606,   608,   612,   616,   620,
     624,   626,   630,   634,   636,   640,   644,   648,   650,   654,
     656,   659,   662,   665,   668,   670,   675,   682,   690,   695,
     701,   705,   706,   709,   713,   723,   736,   747,   749,   753,
     755,   757,   759,   761,   763,   765,   767,   769,   771,   773,
     775,   777,   780,   784,   787,   790,   794,   799,   802,   806,
     808,   812,   814,   818,   821,   825,   832,   834,   837,   842,
     848,   854,   860
};

/* YYRHS -- A `-1'-separated list of the rules' RHS.  */
static const yytype_int16 yyrhs[] =
{
     102,     0,    -1,    -1,   103,    -1,   104,    -1,   103,   104,
      -1,    58,   145,    -1,   118,    -1,   136,    -1,   123,    -1,
     137,    -1,   138,    -1,   129,    -1,   117,    -1,   105,    -1,
     110,    -1,   111,    -1,   112,    -1,   113,    -1,   114,    -1,
     115,    -1,   120,    -1,   141,    -1,   142,    -1,   139,    -1,
     116,    -1,   133,    -1,   134,    -1,   140,    -1,     8,     6,
     106,   108,   107,   143,    -1,    -1,    11,   162,    -1,    11,
       6,    -1,    -1,    13,   109,    -1,    -1,    12,   109,    -1,
       6,    -1,   162,    -1,   109,    59,     6,    -1,   109,    59,
     162,    -1,    58,    17,    56,     6,    57,    -1,    17,    56,
       6,    57,    52,    53,   143,    -1,    58,    18,    56,     6,
      57,    -1,    18,    56,     6,    57,    52,     6,    53,   143,
      -1,    58,    19,    56,     6,    57,    -1,    58,    16,    56,
       6,    57,    -1,    58,    16,    56,    34,    44,     6,    57,
      -1,    58,    15,    56,     6,    57,    -1,    58,    15,    56,
      34,    44,     6,    57,    -1,    58,    14,    56,     6,    57,
      -1,    58,    14,    56,    34,    44,     6,    57,    -1,    10,
       6,   107,   143,    -1,     9,     6,   108,   143,    -1,   119,
     143,    -1,   119,   143,    35,   143,    36,   143,    -1,     7,
       6,    52,    53,    -1,     7,     6,    52,   165,    53,    -1,
       7,     6,    52,   165,    59,    66,     6,    53,    -1,     7,
       6,    52,    66,     6,    53,    -1,     7,    34,    44,     6,
      52,    53,    -1,     7,    34,    44,     6,    52,   165,    53,
      -1,     7,    34,    44,     6,    52,   165,    59,    66,     6,
      53,    -1,     7,    34,    44,     6,    52,    66,     6,    53,
      -1,   121,   143,    -1,     7,   122,    52,    53,    -1,     7,
     122,    52,   165,    53,    -1,    56,    57,    -1,    56,    57,
      63,    -1,    60,    -1,    61,    -1,    62,    -1,    84,    -1,
      85,    -1,    86,    -1,    87,    -1,    88,    -1,    89,    -1,
      64,    -1,    65,    -1,    66,    -1,    67,    -1,    68,    -1,
      69,    -1,    70,    -1,    71,    -1,    72,    -1,    73,    -1,
      83,    -1,    90,    -1,    91,    -1,    92,    -1,   124,    -1,
     127,    -1,    20,    52,   145,    53,   143,    -1,    20,    52,
     145,    53,   143,    21,   143,    -1,    20,    52,   145,    53,
     143,   125,    -1,    20,    52,   145,    53,   143,   125,    21,
     143,    -1,   126,    -1,   125,   126,    -1,    22,    52,   145,
      53,   143,    -1,    20,    52,   145,    59,   145,   128,    53,
     143,    -1,    -1,    59,     6,    -1,    25,    52,   145,    53,
     130,    -1,    54,   131,    55,    -1,    54,   131,    21,   143,
      55,    -1,   132,    -1,   131,   132,    -1,    26,    52,   166,
      53,   143,    -1,    23,    52,     6,    24,   145,    53,   143,
      -1,    23,    52,    52,     6,    59,     6,    53,    24,   145,
      53,   143,    -1,    37,   143,    38,    52,     6,    53,   143,
     135,    -1,    -1,    39,    54,   144,    55,    -1,    58,    32,
      -1,    58,    32,   145,    -1,    58,    30,    -1,    58,    31,
      -1,    58,     7,     6,    52,    53,    -1,    58,     7,     6,
      52,   165,    53,    -1,    58,     7,     6,    52,   165,    59,
      66,     6,    53,    -1,    58,     7,     6,    52,    66,     6,
      53,    -1,    58,    40,    52,   145,    53,    -1,    58,    33,
      -1,    58,    29,    52,    53,    -1,    58,    29,    52,   166,
      53,    -1,    54,    55,    -1,    54,   144,    55,    -1,   104,
      -1,   144,   104,    -1,   147,    -1,   159,   146,   145,    -1,
      63,    -1,    74,    -1,    75,    -1,    76,    -1,    77,    -1,
      78,    -1,    79,    -1,    80,    -1,    81,    -1,    82,    -1,
      93,    -1,    94,    -1,    95,    -1,    96,    -1,   148,    -1,
     147,    61,   148,    -1,   149,    -1,   148,    60,   149,    -1,
     150,    -1,   149,    71,   150,    -1,   151,    -1,   150,    72,
     151,    -1,   152,    -1,   151,    70,   152,    -1,   153,    -1,
     152,    84,   153,    -1,   152,    85,   153,    -1,   154,    -1,
     153,    86,   154,    -1,   153,    87,   154,    -1,   153,    88,
     154,    -1,   153,    89,   154,    -1,   155,    -1,   154,    83,
     155,    -1,   154,    90,   155,    -1,   154,    91,   155,    -1,
     154,    92,   155,    -1,   156,    -1,   155,    64,   156,    -1,
     155,    65,   156,    -1,   157,    -1,   156,    66,   157,    -1,
     156,    67,   157,    -1,   156,    68,   157,    -1,   158,    -1,
     157,    69,   158,    -1,   159,    -1,    62,   158,    -1,    73,
     158,    -1,    65,   158,    -1,    64,   158,    -1,   161,    -1,
     159,    56,   145,    57,    -1,   159,    44,     6,    52,    53,
     160,    -1,   159,    44,     6,    52,   166,    53,   160,    -1,
       6,    52,    53,   160,    -1,     6,    52,   166,    53,   160,
      -1,   159,    44,     6,    -1,    -1,    54,    55,    -1,    54,
     144,    55,    -1,    54,    28,    97,    56,   165,    57,   100,
     144,    55,    -1,    54,    28,    97,    56,   165,    59,    66,
       6,    57,   100,   144,    55,    -1,    54,    28,    97,    56,
      66,     6,    57,   100,   144,    55,    -1,   162,    -1,    52,
     145,    53,    -1,     6,    -1,     3,    -1,     4,    -1,     5,
      -1,    41,    -1,    42,    -1,    43,    -1,    34,    -1,    27,
      -1,   163,    -1,   167,    -1,   170,    -1,   164,     6,    -1,
      98,   164,     6,    -1,    98,     6,    -1,    56,    57,    -1,
      56,   166,    57,    -1,    56,   166,    59,    57,    -1,     6,
      98,    -1,   164,     6,    98,    -1,     6,    -1,   165,    59,
       6,    -1,   145,    -1,   166,    59,   145,    -1,    54,    55,
      -1,    54,   168,    55,    -1,    54,   168,   145,    97,   145,
      55,    -1,   169,    -1,   168,   169,    -1,   145,    97,   145,
      59,    -1,    52,   145,    97,   145,    53,    -1,    52,   145,
      97,   145,    57,    -1,    56,   145,    97,   145,    57,    -1,
      56,   145,    97,   145,    53,    -1
};

/* YYRLINE[YYN] -- source line where rule number YYN was defined.  */
static const yytype_uint16 yyrline[] =
{
       0,   106,   106,   108,   112,   115,   121,   126,   127,   128,
     129,   130,   131,   132,   133,   134,   135,   136,   137,   138,
     139,   140,   141,   142,   143,   144,   145,   146,   147,   152,
     160,   163,   166,   172,   175,   181,   184,   190,   195,   200,
     204,   212,   217,   226,   231,   240,   249,   254,   262,   267,
     275,   280,   288,   297,   306,   311,   319,   322,   325,   328,
     331,   334,   337,   340,   346,   354,   357,   368,   371,   374,
     377,   380,   383,   386,   389,   392,   395,   398,   401,   404,
     407,   410,   413,   416,   419,   422,   425,   428,   431,   434,
     437,   440,   447,   451,   458,   462,   467,   471,   478,   483,
     490,   497,   504,   507,   514,   522,   526,   533,   538,   545,
     553,   558,   567,   575,   578,   586,   591,   600,   608,   617,
     622,   627,   632,   641,   650,   659,   664,   672,   676,   682,
     687,   695,   696,   703,   706,   709,   712,   715,   718,   721,
     724,   727,   730,   733,   736,   739,   742,   748,   749,   756,
     757,   764,   765,   772,   773,   780,   781,   788,   789,   793,
     800,   801,   805,   809,   813,   820,   821,   825,   829,   833,
     840,   841,   845,   852,   853,   857,   861,   868,   869,   876,
     877,   881,   885,   889,   896,   897,   901,   905,   909,   913,
     917,   924,   927,   930,   933,   936,   939,   945,   946,   949,
     953,   957,   961,   965,   969,   973,   977,   981,   985,   986,
     987,   991,   995,   999,  1006,  1010,  1014,  1022,  1027,  1035,
    1040,  1047,  1052,  1059,  1063,  1067,  1074,  1079,  1085,  1090,
    1094,  1098,  1102
};
#endif

#if YYDEBUG || YYERROR_VERBOSE || 0
/* YYTNAME[SYMBOL-NUM] -- String name of the symbol SYMBOL-NUM.
   First, the terminals, then, starting at YYNTOKENS, nonterminals.  */
static const char *const yytname[] =
{
  "$end", "error", "$undefined", "INTEGER", "FLOAT", "STRING",
  "IDENTIFIER", "FUNCTION", "CLASS", "MODULE", "INTERFACE", "EXTENDS",
  "INVOLVES", "JOINTS", "PERSONAL", "RELATIVE", "EVERYONE", "GET", "SET",
  "GSET", "IF", "ELSE", "ELSEIF", "FOR", "IN", "SWITCH", "WHEN", "CAST",
  "ITERATOR", "SUPER", "BREAK", "CONTINUE", "RETURN", "BLOCK", "SELF",
  "WITH", "WITHOUT", "ORDER", "SERVE", "IGNORE", "GROAN", "TRUE", "FALSE",
  "NIL", "DOT", "ALIAS", "RETRY", "REDO", "GOTO", "CONST", "GLOBAL",
  "STATIC", "LP", "RP", "LC", "RC", "LB", "RB", "SEMICOLON", "COMMA",
  "LOGICAL_AND", "LOGICAL_OR", "LOGICAL_NOT", "ASSIGN", "ADD", "SUB",
  "MUL", "DIV", "MOD", "POWER", "BIT_AND", "BIT_OR", "BIT_XOR", "BIT_NOT",
  "ASSIGN_ADD", "ASSIGN_SUB", "ASSIGN_MUL", "ASSIGN_DIV", "ASSIGN_MOD",
  "ASSIGN_POWER", "ASSIGN_BIT_AND", "ASSIGN_BIT_OR", "ASSIGN_BIT_XOR",
  "BIT_SHR", "EQ", "NE", "GT", "GE", "LT", "LE", "BIT_SHL", "BIT_SAR",
  "BIT_SAL", "ASSIGN_BIT_SHR", "ASSIGN_BIT_SHL", "ASSIGN_BIT_SAR",
  "ASSIGN_BIT_SAL", "ITER", "FILED", "SHARP", "CONLON", "$accept", "start",
  "translation_unit", "statement", "class_statement", "extends_field",
  "interface_field", "module_field", "field_identifier_list",
  "getter_statement", "setter_statement", "getter_setter_statement",
  "everyone_statement", "relative_statement", "personal_statement",
  "interface_statement", "module_statement", "function_statement",
  "function_header", "function_operator_statement",
  "function_operator_header", "operator", "if_statement", "condition_if",
  "elseif_list", "elseif", "loop_if", "loop_time", "switch_statement",
  "switch_block", "when_list", "when", "for_statement", "order_statement",
  "ignore_block", "return_statement", "break_statement",
  "continue_statement", "interface_func_statement", "groan_statement",
  "block_statement", "super_statement", "block", "statement_list",
  "expression", "assign_symbol", "logic_or_expression",
  "logic_and_expression", "logic_bit_or_expression",
  "logic_bit_xor_expression", "logic_bit_and_expression",
  "logic_equal_compare_expression", "logic_not_equal_expression",
  "logic_shift_expression", "add_sub_expression", "mul_div_mod_expression",
  "power_expression", "unary_expression", "top_expression",
  "closure_block", "primary_expression", "identifier_with_field",
  "array_literal", "field_identifier", "identifier_list",
  "expression_list", "hash_literal", "hash_pair_list", "hash_pair",
  "range_literal", YY_NULL
};
#endif

# ifdef YYPRINT
/* YYTOKNUM[YYLEX-NUM] -- Internal token number corresponding to
   token YYLEX-NUM.  */
static const yytype_uint16 yytoknum[] =
{
       0,   256,   257,   258,   259,   260,   261,   262,   263,   264,
     265,   266,   267,   268,   269,   270,   271,   272,   273,   274,
     275,   276,   277,   278,   279,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   298,   299,   300,   301,   302,   303,   304,
     305,   306,   307,   308,   309,   310,   311,   312,   313,   314,
     315,   316,   317,   318,   319,   320,   321,   322,   323,   324,
     325,   326,   327,   328,   329,   330,   331,   332,   333,   334,
     335,   336,   337,   338,   339,   340,   341,   342,   343,   344,
     345,   346,   347,   348,   349,   350,   351,   352,   353,   354,
     355
};
# endif

/* YYR1[YYN] -- Symbol number of symbol that rule YYN derives.  */
static const yytype_uint8 yyr1[] =
{
       0,   101,   102,   102,   103,   103,   104,   104,   104,   104,
     104,   104,   104,   104,   104,   104,   104,   104,   104,   104,
     104,   104,   104,   104,   104,   104,   104,   104,   104,   105,
     106,   106,   106,   107,   107,   108,   108,   109,   109,   109,
     109,   110,   110,   111,   111,   112,   113,   113,   114,   114,
     115,   115,   116,   117,   118,   118,   119,   119,   119,   119,
     119,   119,   119,   119,   120,   121,   121,   122,   122,   122,
     122,   122,   122,   122,   122,   122,   122,   122,   122,   122,
     122,   122,   122,   122,   122,   122,   122,   122,   122,   122,
     122,   122,   123,   123,   124,   124,   124,   124,   125,   125,
     126,   127,   128,   128,   129,   130,   130,   131,   131,   132,
     133,   133,   134,   135,   135,   136,   136,   137,   138,   139,
     139,   139,   139,   140,   141,   142,   142,   143,   143,   144,
     144,   145,   145,   146,   146,   146,   146,   146,   146,   146,
     146,   146,   146,   146,   146,   146,   146,   147,   147,   148,
     148,   149,   149,   150,   150,   151,   151,   152,   152,   152,
     153,   153,   153,   153,   153,   154,   154,   154,   154,   154,
     155,   155,   155,   156,   156,   156,   156,   157,   157,   158,
     158,   158,   158,   158,   159,   159,   159,   159,   159,   159,
     159,   160,   160,   160,   160,   160,   160,   161,   161,   161,
     161,   161,   161,   161,   161,   161,   161,   161,   161,   161,
     161,   162,   162,   162,   163,   163,   163,   164,   164,   165,
     165,   166,   166,   167,   167,   167,   168,   168,   169,   170,
     170,   170,   170
};

/* YYR2[YYN] -- Number of symbols composing right hand side of rule YYN.  */
static const yytype_uint8 yyr2[] =
{
       0,     2,     0,     1,     1,     2,     2,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     6,
       0,     2,     2,     0,     2,     0,     2,     1,     1,     3,
       3,     5,     7,     5,     8,     5,     5,     7,     5,     7,
       5,     7,     4,     4,     2,     6,     4,     5,     8,     6,
       6,     7,    10,     8,     2,     4,     5,     2,     3,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     5,     7,     6,     8,     1,     2,
       5,     8,     0,     2,     5,     3,     5,     1,     2,     5,
       7,    11,     8,     0,     4,     2,     3,     2,     2,     5,
       6,     9,     7,     5,     2,     4,     5,     2,     3,     1,
       2,     1,     3,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     3,     1,
       3,     1,     3,     1,     3,     1,     3,     1,     3,     3,
       1,     3,     3,     3,     3,     1,     3,     3,     3,     3,
       1,     3,     3,     1,     3,     3,     3,     1,     3,     1,
       2,     2,     2,     2,     1,     4,     6,     7,     4,     5,
       3,     0,     2,     3,     9,    12,    10,     1,     3,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     2,     3,     2,     2,     3,     4,     2,     3,     1,
       3,     1,     3,     2,     3,     6,     1,     2,     4,     5,
       5,     5,     5
};

/* YYDEFACT[STATE-NAME] -- Default reduction number in state STATE-NUM.
   Performed when YYTABLE doesn't specify something else to do.  Zero
   means the default is an error.  */
static const yytype_uint8 yydefact[] =
{
       2,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     3,     4,    14,    15,    16,    17,    18,
      19,    20,    25,    13,     7,     0,    21,     0,     9,    92,
      93,    12,    26,    27,     8,    10,    11,    24,    28,    22,
      23,     0,     0,     0,    69,    70,    71,    78,    79,    80,
      81,    82,    83,    84,    85,    86,    87,    88,    72,    73,
      74,    75,    76,    77,    89,    90,    91,     0,    30,    35,
      33,     0,     0,     0,     0,     0,     0,     0,   200,   201,
     202,   199,     0,     0,     0,     0,     0,     0,     0,   207,
       0,   117,   118,   115,   124,   206,     0,   203,   204,   205,
       0,     0,     0,     0,     0,     0,     0,     0,     6,   131,
     147,   149,   151,   153,   155,   157,   160,   165,   170,   173,
     177,   179,   184,   197,   208,     0,   209,   210,     1,     5,
      54,    64,     0,     0,    67,     0,     0,    35,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,   127,   129,
       0,     0,     0,   217,     0,     0,     0,     0,     0,     0,
       0,     0,   116,     0,     0,   223,     0,     0,   226,   214,
     221,     0,   180,   179,   183,   182,   181,   213,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,   133,   134,   135,   136,   137,   138,   139,   140,
     141,   142,   143,   144,   145,   146,     0,   211,     0,   219,
      56,     0,     0,     0,    68,    65,     0,    32,    31,    33,
      37,    36,    38,    53,    34,    52,     0,     0,     0,     0,
       0,     0,     0,   128,   130,     0,   191,   221,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,   125,
       0,     0,   198,     0,     0,   224,     0,   227,     0,   215,
       0,   212,   148,   150,   152,   154,   156,   158,   159,   161,
     162,   163,   164,   166,   167,   168,   169,   171,   172,   174,
     175,   176,   178,   190,     0,   132,   218,     0,     0,    57,
       0,     0,    66,     0,     0,     0,     0,     0,    94,   102,
       0,     0,     0,   104,     0,     0,   188,   191,     0,   119,
       0,     0,    50,     0,    48,     0,    46,     0,    41,    43,
      45,   126,   123,     0,     0,     0,     0,   216,   222,     0,
     185,     0,    59,   220,     0,    60,     0,     0,    29,    39,
      40,     0,     0,     0,     0,    96,    98,     0,     0,     0,
       0,     0,     0,   107,     0,     0,   192,     0,   189,     0,
     120,     0,     0,     0,     0,   229,   230,   228,     0,   232,
     231,   191,     0,    55,     0,     0,    61,     0,    42,     0,
      95,     0,     0,    99,   103,     0,   110,     0,     0,     0,
     105,   108,   113,     0,   193,   122,     0,    51,    49,    47,
     225,   186,   191,    58,    63,     0,    44,     0,    97,   101,
       0,     0,     0,     0,   112,     0,     0,   187,     0,     0,
       0,     0,   106,     0,     0,     0,   121,    62,   100,     0,
     109,     0,     0,     0,     0,   111,   114,     0,     0,     0,
       0,     0,     0,     0,   194,     0,   196,     0,     0,   195
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
      -1,    12,    13,   149,    15,   137,   141,   139,   231,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    67,    28,    29,   355,   356,    30,   358,    31,   313,
     362,   363,    32,    33,   424,    34,    35,    36,    37,    38,
      39,    40,    77,   150,   247,   216,   109,   110,   111,   112,
     113,   114,   115,   116,   117,   118,   119,   120,   121,   316,
     122,   123,   124,   125,   222,   171,   126,   167,   168,   127
};

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
#define YYPACT_NINF -311
static const yytype_int16 yypact[] =
{
     810,   641,    36,    52,    61,   -25,    13,    29,    64,    83,
      88,   206,   144,   810,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,    88,  -311,    88,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,   104,   127,   117,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,  -311,  -311,   130,   180,   190,
     193,   202,   213,   625,    27,   625,    28,   194,  -311,  -311,
    -311,   -18,   228,   187,   188,   189,   195,   197,   200,  -311,
     198,  -311,  -311,   625,  -311,  -311,   211,  -311,  -311,  -311,
     625,   296,   375,   625,   625,   625,   625,   251,  -311,   203,
     207,   201,   209,   199,   131,    97,   -29,    25,   133,   208,
    -311,    85,  -311,  -311,  -311,   259,  -311,  -311,  -311,  -311,
     231,  -311,     2,   272,   219,    22,    23,   190,    24,    88,
      24,    88,   230,   233,    72,   261,   282,   239,  -311,  -311,
     266,   241,   418,  -311,   255,    37,    44,    54,   304,   306,
     308,   459,  -311,   625,   -12,  -311,   218,   500,  -311,  -311,
     221,   -10,  -311,    -4,  -311,  -311,  -311,   222,   313,   625,
     625,   625,   625,   625,   625,   625,   625,   625,   625,   625,
     625,   625,   625,   625,   625,   625,   625,   625,   625,   625,
     316,   625,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,  -311,   625,   234,    88,  -311,
    -311,   322,    73,   279,  -311,  -311,    74,   222,  -311,   193,
     222,   274,  -311,  -311,   274,  -311,   284,   289,    88,   625,
     625,   275,   290,  -311,  -311,   336,   291,  -311,    75,     4,
     292,   303,   297,   309,   298,   312,   300,   302,   305,  -311,
      77,   310,  -311,   625,   625,  -311,   267,  -311,   625,  -311,
     543,   234,   207,   201,   209,   199,   131,    97,    97,   -29,
     -29,   -29,   -29,    25,    25,    25,    25,   133,   133,   208,
     208,   208,  -311,   314,   315,  -311,  -311,   334,   320,  -311,
       8,     6,  -311,   368,    88,    26,   323,   371,   196,   324,
     331,   379,   360,  -311,   335,   288,  -311,   291,   625,  -311,
     381,    84,  -311,   383,  -311,   384,  -311,   385,  -311,  -311,
    -311,  -311,  -311,   100,   333,   625,   115,  -311,  -311,   584,
    -311,    88,  -311,  -311,   387,  -311,   389,    86,  -311,   222,
    -311,    88,   343,    88,   345,   205,  -311,   392,   346,    88,
     347,   349,    18,  -311,    88,   307,  -311,   727,  -311,   350,
    -311,    10,   351,   353,   355,  -311,  -311,  -311,   114,  -311,
    -311,   291,    93,  -311,   361,   366,  -311,    11,  -311,    88,
    -311,   625,    88,  -311,  -311,    88,  -311,   382,   625,    88,
    -311,  -311,   374,   364,  -311,  -311,   419,  -311,  -311,  -311,
    -311,  -311,   291,  -311,  -311,   420,  -311,   377,  -311,  -311,
     625,    96,   373,   380,  -311,    16,   390,  -311,   391,    88,
     393,    88,  -311,   810,   427,   118,  -311,  -311,  -311,    88,
    -311,   731,   378,   338,    21,  -311,  -311,   342,   810,   430,
     810,   753,   397,   784,  -311,   356,  -311,   810,   806,  -311
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -311,  -311,  -311,     0,  -311,  -311,   220,   318,   317,  -311,
    -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,  -311,  -311,  -311,    92,  -311,  -311,  -311,  -311,
    -311,   105,  -311,  -311,  -311,  -311,  -311,  -311,  -311,  -311,
    -311,  -311,   -24,  -310,    -9,  -311,  -311,   271,   286,   287,
     293,   294,    57,     1,     5,    34,     7,   -80,   -85,  -261,
    -311,  -129,  -311,   362,  -131,  -146,  -311,  -311,   311,  -311
};

/* YYTABLE[YYPACT[STATE-NUM]].  What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule which
   number is the opposite.  If YYTABLE_NINF, syntax error.  */
#define YYTABLE_NINF -1
static const yytype_uint16 yytable[] =
{
      14,   130,   108,   131,   226,   367,   248,   228,   219,   232,
     219,   232,   219,   129,   343,   260,   343,   343,   173,   173,
     173,   173,   219,   172,   174,   175,   176,   343,   219,   227,
     230,    71,   349,   145,   152,     1,     2,     3,     4,   399,
     200,   262,    68,   250,   361,     5,     6,   269,     7,   270,
     252,     8,   201,     9,   190,   220,   368,   319,    69,   345,
     254,   191,   192,   193,   144,    10,   147,    70,   221,    72,
     320,   251,   346,   400,   344,   225,   406,   415,   253,   146,
     153,    73,   434,   148,   162,   263,    11,   449,   255,   194,
     195,   164,   166,   170,   173,   173,   173,   173,   173,   173,
     173,   173,   173,   173,   173,   173,   173,   173,   173,   173,
     173,   173,   173,   173,   173,   233,    74,   235,   321,   292,
     411,   107,   107,   441,   107,   238,   299,   302,   317,   200,
     331,   239,   300,   303,   318,    75,   318,   370,   451,   386,
     453,   201,    76,   371,   128,   387,   412,   458,   202,   431,
     244,   427,   318,   375,   261,   318,   132,   376,   266,   203,
     204,   205,   206,   207,   208,   209,   210,   211,   379,   410,
     347,   133,   380,   377,   134,   443,   350,   444,   212,   213,
     214,   215,   135,   186,   187,   188,   189,   279,   280,   281,
     282,   136,   294,   382,   297,   283,   284,   285,   286,   196,
     197,   198,   138,   289,   290,   291,   140,   295,   142,    78,
      79,    80,    81,    82,   308,   184,   185,   353,   354,   143,
      83,    84,    85,    86,    87,    88,   392,   354,   287,   288,
     309,   310,   151,    89,   154,    90,    91,    92,    93,    94,
      95,   277,   278,   155,   156,   157,    96,    97,    98,    99,
     161,   158,   421,   159,   333,   334,   160,   177,   100,   336,
     101,   338,   102,   163,   179,   217,   218,   180,   103,   183,
     104,   105,   181,     1,     2,     3,     4,   199,   223,   106,
     348,   182,   224,     5,     6,   240,     7,   236,   241,     8,
     237,     9,   242,   245,   435,     1,     2,     3,     4,    78,
      79,    80,    81,    10,   107,     5,     6,   249,     7,   338,
     256,     8,   257,     9,   258,   264,   365,   383,   268,   271,
     153,   243,   293,    89,    11,    10,   378,   388,   298,   390,
      95,   301,   296,   305,   311,   396,   306,    97,    98,    99,
     402,   307,   314,   366,   312,   315,    11,   323,   100,   322,
     101,   165,   102,   325,   324,   326,   327,   328,   103,   329,
     104,   105,   330,   332,   335,   416,   339,   244,   418,   106,
     341,   419,   340,   342,   343,   422,   351,   352,    78,    79,
      80,    81,   417,   357,   359,   360,   361,   369,   364,   372,
     373,   374,   377,   384,   107,   385,   389,   391,   394,   395,
     397,   398,    89,   405,   403,   438,   420,   440,   407,    95,
     408,   430,   409,   423,   413,   445,    97,    98,    99,   414,
     425,    78,    79,    80,    81,   426,   428,   100,   432,   101,
     429,   102,   169,   442,   433,   447,   452,   103,   448,   104,
     105,   244,   450,   436,   437,    89,   439,   393,   106,   304,
     272,   244,    95,   244,   455,   229,   457,   234,   244,    97,
      98,    99,    78,    79,    80,    81,   273,   401,   274,   178,
     100,   246,   101,   107,   102,   275,     0,   276,   267,     0,
     103,     0,   104,   105,     0,     0,    89,     0,     0,     0,
       0,   106,     0,    95,     0,     0,     0,     0,     0,     0,
      97,    98,    99,    78,    79,    80,    81,     0,     0,     0,
       0,   100,   259,   101,     0,   102,   107,     0,     0,     0,
       0,   103,     0,   104,   105,     0,     0,    89,     0,     0,
       0,     0,   106,     0,    95,     0,     0,     0,     0,     0,
       0,    97,    98,    99,     0,     0,    78,    79,    80,    81,
       0,     0,   100,     0,   101,   265,   102,   107,     0,     0,
       0,     0,   103,     0,   104,   105,     0,     0,     0,     0,
      89,     0,     0,   106,     0,     0,     0,    95,     0,     0,
       0,     0,     0,     0,    97,    98,    99,    78,    79,    80,
      81,     0,     0,     0,     0,   100,     0,   101,   107,   102,
     337,     0,     0,     0,     0,   103,     0,   104,   105,     0,
       0,    89,     0,     0,     0,     0,   106,     0,    95,     0,
       0,     0,     0,     0,     0,    97,    98,    99,    78,    79,
      80,    81,     0,     0,     0,     0,   100,   381,   101,     0,
     102,   107,     0,     0,     0,     0,   103,    41,   104,   105,
       0,     0,    89,     0,     0,     0,     0,   106,     0,    95,
       0,     0,     0,     0,     0,     0,    97,    98,    99,     0,
       0,     0,     0,     0,     0,    42,     0,   100,     0,   101,
       0,   102,   107,     0,     0,     0,     0,   103,     0,   104,
     105,     0,     0,     0,     0,     0,     0,    43,   106,     0,
       0,    44,    45,    46,     0,    47,    48,    49,    50,    51,
      52,    53,    54,    55,    56,     0,     0,     0,     0,     0,
       0,     0,     0,   107,    57,    58,    59,    60,    61,    62,
      63,    64,    65,    66,     1,     2,     3,     4,     1,     2,
       3,     4,     0,     0,     5,     6,     0,     7,     5,     6,
       8,     7,     9,     0,     8,     0,     9,     0,     0,     0,
       1,     2,     3,     4,    10,     0,     0,     0,    10,     0,
       5,     6,     0,     7,     0,     0,     8,     0,     9,     0,
       0,     0,   404,     0,     0,    11,   446,     0,     0,    11,
      10,     1,     2,     3,     4,     0,     0,     0,     0,     0,
       0,     5,     6,     0,     7,     0,     0,     8,   454,     9,
       0,    11,     0,     1,     2,     3,     4,     1,     2,     3,
       4,    10,     0,     5,     6,     0,     7,     5,     6,     8,
       7,     9,     0,     8,     0,     9,     0,     0,     0,   456,
       0,     0,    11,    10,     0,     0,     0,    10,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,   459,     0,     0,    11,     0,     0,     0,    11
};

#define yypact_value_is_default(Yystate) \
  (!!((Yystate) == (-311)))

#define yytable_value_is_error(Yytable_value) \
  YYID (0)

static const yytype_int16 yycheck[] =
{
       0,    25,    11,    27,   135,   315,   152,   136,     6,   138,
       6,   140,     6,    13,     6,   161,     6,     6,   103,   104,
     105,   106,     6,   103,   104,   105,   106,     6,     6,     6,
       6,    56,     6,     6,    52,     7,     8,     9,    10,    21,
      44,    53,     6,     6,    26,    17,    18,    57,    20,    59,
       6,    23,    56,    25,    83,    53,   317,    53,     6,    53,
       6,    90,    91,    92,    73,    37,    75,     6,    66,    56,
      66,    34,    66,    55,    66,    53,    66,    66,    34,    52,
      98,    52,    66,    55,    93,    97,    58,    66,    34,    64,
      65,   100,   101,   102,   179,   180,   181,   182,   183,   184,
     185,   186,   187,   188,   189,   190,   191,   192,   193,   194,
     195,   196,   197,   198,   199,   139,    52,   141,   249,   199,
     381,    98,    98,   433,    98,    53,    53,    53,    53,    44,
      53,    59,    59,    59,    59,    52,    59,    53,   448,    53,
     450,    56,    54,    59,     0,    59,    53,   457,    63,    53,
     150,   412,    59,    53,   163,    59,    52,    57,   167,    74,
      75,    76,    77,    78,    79,    80,    81,    82,    53,    55,
     301,    44,    57,    59,    57,    57,   305,    59,    93,    94,
      95,    96,    52,    86,    87,    88,    89,   186,   187,   188,
     189,    11,   201,   339,   218,   190,   191,   192,   193,    66,
      67,    68,    12,   196,   197,   198,    13,   216,     6,     3,
       4,     5,     6,     7,   238,    84,    85,    21,    22,     6,
      14,    15,    16,    17,    18,    19,    21,    22,   194,   195,
     239,   240,    38,    27,     6,    29,    30,    31,    32,    33,
      34,   184,   185,    56,    56,    56,    40,    41,    42,    43,
      52,    56,   398,    56,   263,   264,    56,     6,    52,   268,
      54,   270,    56,    52,    61,     6,    35,    60,    62,    70,
      64,    65,    71,     7,     8,     9,    10,    69,     6,    73,
     304,    72,    63,    17,    18,    24,    20,    57,     6,    23,
      57,    25,    53,    52,   425,     7,     8,     9,    10,     3,
       4,     5,     6,    37,    98,    17,    18,    52,    20,   318,
       6,    23,     6,    25,     6,    97,    28,   341,    97,     6,
      98,    55,     6,    27,    58,    37,   335,   351,     6,   353,
      34,    52,    98,    59,    59,   359,    52,    41,    42,    43,
     364,    52,     6,    55,    54,    54,    58,    44,    52,    57,
      54,    55,    56,    44,    57,    57,    44,    57,    62,    57,
      64,    65,    57,    53,    97,   389,    52,   367,   392,    73,
      36,   395,    57,    53,     6,   399,    53,     6,     3,     4,
       5,     6,   391,    59,    53,     6,    26,     6,    53,     6,
       6,     6,    59,     6,    98,     6,    53,    52,     6,    53,
      53,    52,    27,    53,    97,   429,    24,   431,    57,    34,
      57,   420,    57,    39,    53,   439,    41,    42,    43,    53,
      56,     3,     4,     5,     6,     6,     6,    52,    55,    54,
      53,    56,    57,     6,    54,    57,     6,    62,   100,    64,
      65,   441,   100,    53,    53,    27,    53,   355,    73,   229,
     179,   451,    34,   453,    57,   137,   100,   140,   458,    41,
      42,    43,     3,     4,     5,     6,   180,   362,   181,   107,
      52,    53,    54,    98,    56,   182,    -1,   183,   167,    -1,
      62,    -1,    64,    65,    -1,    -1,    27,    -1,    -1,    -1,
      -1,    73,    -1,    34,    -1,    -1,    -1,    -1,    -1,    -1,
      41,    42,    43,     3,     4,     5,     6,    -1,    -1,    -1,
      -1,    52,    53,    54,    -1,    56,    98,    -1,    -1,    -1,
      -1,    62,    -1,    64,    65,    -1,    -1,    27,    -1,    -1,
      -1,    -1,    73,    -1,    34,    -1,    -1,    -1,    -1,    -1,
      -1,    41,    42,    43,    -1,    -1,     3,     4,     5,     6,
      -1,    -1,    52,    -1,    54,    55,    56,    98,    -1,    -1,
      -1,    -1,    62,    -1,    64,    65,    -1,    -1,    -1,    -1,
      27,    -1,    -1,    73,    -1,    -1,    -1,    34,    -1,    -1,
      -1,    -1,    -1,    -1,    41,    42,    43,     3,     4,     5,
       6,    -1,    -1,    -1,    -1,    52,    -1,    54,    98,    56,
      57,    -1,    -1,    -1,    -1,    62,    -1,    64,    65,    -1,
      -1,    27,    -1,    -1,    -1,    -1,    73,    -1,    34,    -1,
      -1,    -1,    -1,    -1,    -1,    41,    42,    43,     3,     4,
       5,     6,    -1,    -1,    -1,    -1,    52,    53,    54,    -1,
      56,    98,    -1,    -1,    -1,    -1,    62,     6,    64,    65,
      -1,    -1,    27,    -1,    -1,    -1,    -1,    73,    -1,    34,
      -1,    -1,    -1,    -1,    -1,    -1,    41,    42,    43,    -1,
      -1,    -1,    -1,    -1,    -1,    34,    -1,    52,    -1,    54,
      -1,    56,    98,    -1,    -1,    -1,    -1,    62,    -1,    64,
      65,    -1,    -1,    -1,    -1,    -1,    -1,    56,    73,    -1,
      -1,    60,    61,    62,    -1,    64,    65,    66,    67,    68,
      69,    70,    71,    72,    73,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    98,    83,    84,    85,    86,    87,    88,
      89,    90,    91,    92,     7,     8,     9,    10,     7,     8,
       9,    10,    -1,    -1,    17,    18,    -1,    20,    17,    18,
      23,    20,    25,    -1,    23,    -1,    25,    -1,    -1,    -1,
       7,     8,     9,    10,    37,    -1,    -1,    -1,    37,    -1,
      17,    18,    -1,    20,    -1,    -1,    23,    -1,    25,    -1,
      -1,    -1,    55,    -1,    -1,    58,    55,    -1,    -1,    58,
      37,     7,     8,     9,    10,    -1,    -1,    -1,    -1,    -1,
      -1,    17,    18,    -1,    20,    -1,    -1,    23,    55,    25,
      -1,    58,    -1,     7,     8,     9,    10,     7,     8,     9,
      10,    37,    -1,    17,    18,    -1,    20,    17,    18,    23,
      20,    25,    -1,    23,    -1,    25,    -1,    -1,    -1,    55,
      -1,    -1,    58,    37,    -1,    -1,    -1,    37,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    55,    -1,    -1,    58,    -1,    -1,    -1,    58
};

/* YYSTOS[STATE-NUM] -- The (internal number of the) accessing
   symbol of state STATE-NUM.  */
static const yytype_uint8 yystos[] =
{
       0,     7,     8,     9,    10,    17,    18,    20,    23,    25,
      37,    58,   102,   103,   104,   105,   110,   111,   112,   113,
     114,   115,   116,   117,   118,   119,   120,   121,   123,   124,
     127,   129,   133,   134,   136,   137,   138,   139,   140,   141,
     142,     6,    34,    56,    60,    61,    62,    64,    65,    66,
      67,    68,    69,    70,    71,    72,    73,    83,    84,    85,
      86,    87,    88,    89,    90,    91,    92,   122,     6,     6,
       6,    56,    56,    52,    52,    52,    54,   143,     3,     4,
       5,     6,     7,    14,    15,    16,    17,    18,    19,    27,
      29,    30,    31,    32,    33,    34,    40,    41,    42,    43,
      52,    54,    56,    62,    64,    65,    73,    98,   145,   147,
     148,   149,   150,   151,   152,   153,   154,   155,   156,   157,
     158,   159,   161,   162,   163,   164,   167,   170,     0,   104,
     143,   143,    52,    44,    57,    52,    11,   106,    12,   108,
      13,   107,     6,     6,   145,     6,    52,   145,    55,   104,
     144,    38,    52,    98,     6,    56,    56,    56,    56,    56,
      56,    52,   145,    52,   145,    55,   145,   168,   169,    57,
     145,   166,   158,   159,   158,   158,   158,     6,   164,    61,
      60,    71,    72,    70,    84,    85,    86,    87,    88,    89,
      83,    90,    91,    92,    64,    65,    66,    67,    68,    69,
      44,    56,    63,    74,    75,    76,    77,    78,    79,    80,
      81,    82,    93,    94,    95,    96,   146,     6,    35,     6,
      53,    66,   165,     6,    63,    53,   165,     6,   162,   108,
       6,   109,   162,   143,   109,   143,    57,    57,    53,    59,
      24,     6,    53,    55,   104,    52,    53,   145,   166,    52,
       6,    34,     6,    34,     6,    34,     6,     6,     6,    53,
     166,   145,    53,    97,    97,    55,   145,   169,    97,    57,
      59,     6,   148,   149,   150,   151,   152,   153,   153,   154,
     154,   154,   154,   155,   155,   155,   155,   156,   156,   157,
     157,   157,   158,     6,   145,   145,    98,   143,     6,    53,
      59,    52,    53,    59,   107,    59,    52,    52,   143,   145,
     145,    59,    54,   130,     6,    54,   160,    53,    59,    53,
      66,   165,    57,    44,    57,    44,    57,    44,    57,    57,
      57,    53,    53,   145,   145,    97,   145,    57,   145,    52,
      57,    36,    53,     6,    66,    53,    66,   165,   143,     6,
     162,    53,     6,    21,    22,   125,   126,    59,   128,    53,
       6,    26,   131,   132,    53,    28,    55,   144,   160,     6,
      53,    59,     6,     6,     6,    53,    57,    59,   145,    53,
      57,    53,   166,   143,     6,     6,    53,    59,   143,    53,
     143,    52,    21,   126,     6,    53,   143,    53,    52,    21,
      55,   132,   143,    97,    55,    53,    66,    57,    57,    57,
      55,   160,    53,    53,    53,    66,   143,   145,   143,   143,
      24,   166,   143,    39,   135,    56,     6,   160,     6,    53,
     145,    53,    55,    54,    66,   165,    53,    53,   143,    53,
     143,   144,     6,    57,    59,   143,    55,    57,   100,    66,
     100,   144,     6,   144,    55,    57,    55,   100,   144,    55
};

#define yyerrok		(yyerrstatus = 0)
#define yyclearin	(yychar = YYEMPTY)
#define YYEMPTY		(-2)
#define YYEOF		0

#define YYACCEPT	goto yyacceptlab
#define YYABORT		goto yyabortlab
#define YYERROR		goto yyerrorlab


/* Like YYERROR except do call yyerror.  This remains here temporarily
   to ease the transition to the new meaning of YYERROR, for GCC.
   Once GCC version 2 has supplanted version 1, this can go.  However,
   YYFAIL appears to be in use.  Nevertheless, it is formally deprecated
   in Bison 2.4.2's NEWS entry, where a plan to phase it out is
   discussed.  */

#define YYFAIL		goto yyerrlab
#if defined YYFAIL
  /* This is here to suppress warnings from the GCC cpp's
     -Wunused-macros.  Normally we don't worry about that warning, but
     some users do, and we want to make it easy for users to remove
     YYFAIL uses, which will produce warnings from Bison 2.5.  */
#endif

#define YYRECOVERING()  (!!yyerrstatus)

#define YYBACKUP(Token, Value)                                  \
do                                                              \
  if (yychar == YYEMPTY)                                        \
    {                                                           \
      yychar = (Token);                                         \
      yylval = (Value);                                         \
      YYPOPSTACK (yylen);                                       \
      yystate = *yyssp;                                         \
      goto yybackup;                                            \
    }                                                           \
  else                                                          \
    {                                                           \
      yyerror (YY_("syntax error: cannot back up")); \
      YYERROR;							\
    }								\
while (YYID (0))

/* Error token number */
#define YYTERROR	1
#define YYERRCODE	256


/* This macro is provided for backward compatibility. */
#ifndef YY_LOCATION_PRINT
# define YY_LOCATION_PRINT(File, Loc) ((void) 0)
#endif


/* YYLEX -- calling `yylex' with the right arguments.  */
#ifdef YYLEX_PARAM
# define YYLEX yylex (YYLEX_PARAM)
#else
# define YYLEX yylex ()
#endif

/* Enable debugging if requested.  */
#if YYDEBUG

# ifndef YYFPRINTF
#  include <stdio.h> /* INFRINGES ON USER NAME SPACE */
#  define YYFPRINTF fprintf
# endif

# define YYDPRINTF(Args)			\
do {						\
  if (yydebug)					\
    YYFPRINTF Args;				\
} while (YYID (0))

# define YY_SYMBOL_PRINT(Title, Type, Value, Location)			  \
do {									  \
  if (yydebug)								  \
    {									  \
      YYFPRINTF (stderr, "%s ", Title);					  \
      yy_symbol_print (stderr,						  \
		  Type, Value); \
      YYFPRINTF (stderr, "\n");						  \
    }									  \
} while (YYID (0))


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_value_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_value_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  FILE *yyo = yyoutput;
  YYUSE (yyo);
  if (!yyvaluep)
    return;
# ifdef YYPRINT
  if (yytype < YYNTOKENS)
    YYPRINT (yyoutput, yytoknum[yytype], *yyvaluep);
# else
  YYUSE (yyoutput);
# endif
  switch (yytype)
    {
      default:
        break;
    }
}


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  if (yytype < YYNTOKENS)
    YYFPRINTF (yyoutput, "token %s (", yytname[yytype]);
  else
    YYFPRINTF (yyoutput, "nterm %s (", yytname[yytype]);

  yy_symbol_value_print (yyoutput, yytype, yyvaluep);
  YYFPRINTF (yyoutput, ")");
}

/*------------------------------------------------------------------.
| yy_stack_print -- Print the state stack from its BOTTOM up to its |
| TOP (included).                                                   |
`------------------------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_stack_print (yytype_int16 *yybottom, yytype_int16 *yytop)
#else
static void
yy_stack_print (yybottom, yytop)
    yytype_int16 *yybottom;
    yytype_int16 *yytop;
#endif
{
  YYFPRINTF (stderr, "Stack now");
  for (; yybottom <= yytop; yybottom++)
    {
      int yybot = *yybottom;
      YYFPRINTF (stderr, " %d", yybot);
    }
  YYFPRINTF (stderr, "\n");
}

# define YY_STACK_PRINT(Bottom, Top)				\
do {								\
  if (yydebug)							\
    yy_stack_print ((Bottom), (Top));				\
} while (YYID (0))


/*------------------------------------------------.
| Report that the YYRULE is going to be reduced.  |
`------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_reduce_print (YYSTYPE *yyvsp, int yyrule)
#else
static void
yy_reduce_print (yyvsp, yyrule)
    YYSTYPE *yyvsp;
    int yyrule;
#endif
{
  int yynrhs = yyr2[yyrule];
  int yyi;
  unsigned long int yylno = yyrline[yyrule];
  YYFPRINTF (stderr, "Reducing stack by rule %d (line %lu):\n",
	     yyrule - 1, yylno);
  /* The symbols being reduced.  */
  for (yyi = 0; yyi < yynrhs; yyi++)
    {
      YYFPRINTF (stderr, "   $%d = ", yyi + 1);
      yy_symbol_print (stderr, yyrhs[yyprhs[yyrule] + yyi],
		       &(yyvsp[(yyi + 1) - (yynrhs)])
		       		       );
      YYFPRINTF (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)		\
do {					\
  if (yydebug)				\
    yy_reduce_print (yyvsp, Rule); \
} while (YYID (0))

/* Nonzero means print parse trace.  It is left uninitialized so that
   multiple parsers can coexist.  */
int yydebug;
#else /* !YYDEBUG */
# define YYDPRINTF(Args)
# define YY_SYMBOL_PRINT(Title, Type, Value, Location)
# define YY_STACK_PRINT(Bottom, Top)
# define YY_REDUCE_PRINT(Rule)
#endif /* !YYDEBUG */


/* YYINITDEPTH -- initial size of the parser's stacks.  */
#ifndef	YYINITDEPTH
# define YYINITDEPTH 200
#endif

/* YYMAXDEPTH -- maximum size the stacks can grow to (effective only
   if the built-in stack extension method is used).

   Do not make this value too large; the results are undefined if
   YYSTACK_ALLOC_MAXIMUM < YYSTACK_BYTES (YYMAXDEPTH)
   evaluated with infinite-precision integer arithmetic.  */

#ifndef YYMAXDEPTH
# define YYMAXDEPTH 10000
#endif


#if YYERROR_VERBOSE

# ifndef yystrlen
#  if defined __GLIBC__ && defined _STRING_H
#   define yystrlen strlen
#  else
/* Return the length of YYSTR.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static YYSIZE_T
yystrlen (const char *yystr)
#else
static YYSIZE_T
yystrlen (yystr)
    const char *yystr;
#endif
{
  YYSIZE_T yylen;
  for (yylen = 0; yystr[yylen]; yylen++)
    continue;
  return yylen;
}
#  endif
# endif

# ifndef yystpcpy
#  if defined __GLIBC__ && defined _STRING_H && defined _GNU_SOURCE
#   define yystpcpy stpcpy
#  else
/* Copy YYSRC to YYDEST, returning the address of the terminating '\0' in
   YYDEST.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static char *
yystpcpy (char *yydest, const char *yysrc)
#else
static char *
yystpcpy (yydest, yysrc)
    char *yydest;
    const char *yysrc;
#endif
{
  char *yyd = yydest;
  const char *yys = yysrc;

  while ((*yyd++ = *yys++) != '\0')
    continue;

  return yyd - 1;
}
#  endif
# endif

# ifndef yytnamerr
/* Copy to YYRES the contents of YYSTR after stripping away unnecessary
   quotes and backslashes, so that it's suitable for yyerror.  The
   heuristic is that double-quoting is unnecessary unless the string
   contains an apostrophe, a comma, or backslash (other than
   backslash-backslash).  YYSTR is taken from yytname.  If YYRES is
   null, do not copy; instead, return the length of what the result
   would have been.  */
static YYSIZE_T
yytnamerr (char *yyres, const char *yystr)
{
  if (*yystr == '"')
    {
      YYSIZE_T yyn = 0;
      char const *yyp = yystr;

      for (;;)
	switch (*++yyp)
	  {
	  case '\'':
	  case ',':
	    goto do_not_strip_quotes;

	  case '\\':
	    if (*++yyp != '\\')
	      goto do_not_strip_quotes;
	    /* Fall through.  */
	  default:
	    if (yyres)
	      yyres[yyn] = *yyp;
	    yyn++;
	    break;

	  case '"':
	    if (yyres)
	      yyres[yyn] = '\0';
	    return yyn;
	  }
    do_not_strip_quotes: ;
    }

  if (! yyres)
    return yystrlen (yystr);

  return yystpcpy (yyres, yystr) - yyres;
}
# endif

/* Copy into *YYMSG, which is of size *YYMSG_ALLOC, an error message
   about the unexpected token YYTOKEN for the state stack whose top is
   YYSSP.

   Return 0 if *YYMSG was successfully written.  Return 1 if *YYMSG is
   not large enough to hold the message.  In that case, also set
   *YYMSG_ALLOC to the required number of bytes.  Return 2 if the
   required number of bytes is too large to store.  */
static int
yysyntax_error (YYSIZE_T *yymsg_alloc, char **yymsg,
                yytype_int16 *yyssp, int yytoken)
{
  YYSIZE_T yysize0 = yytnamerr (YY_NULL, yytname[yytoken]);
  YYSIZE_T yysize = yysize0;
  enum { YYERROR_VERBOSE_ARGS_MAXIMUM = 5 };
  /* Internationalized format string. */
  const char *yyformat = YY_NULL;
  /* Arguments of yyformat. */
  char const *yyarg[YYERROR_VERBOSE_ARGS_MAXIMUM];
  /* Number of reported tokens (one for the "unexpected", one per
     "expected"). */
  int yycount = 0;

  /* There are many possibilities here to consider:
     - Assume YYFAIL is not used.  It's too flawed to consider.  See
       <http://lists.gnu.org/archive/html/bison-patches/2009-12/msg00024.html>
       for details.  YYERROR is fine as it does not invoke this
       function.
     - If this state is a consistent state with a default action, then
       the only way this function was invoked is if the default action
       is an error action.  In that case, don't check for expected
       tokens because there are none.
     - The only way there can be no lookahead present (in yychar) is if
       this state is a consistent state with a default action.  Thus,
       detecting the absence of a lookahead is sufficient to determine
       that there is no unexpected or expected token to report.  In that
       case, just report a simple "syntax error".
     - Don't assume there isn't a lookahead just because this state is a
       consistent state with a default action.  There might have been a
       previous inconsistent state, consistent state with a non-default
       action, or user semantic action that manipulated yychar.
     - Of course, the expected token list depends on states to have
       correct lookahead information, and it depends on the parser not
       to perform extra reductions after fetching a lookahead from the
       scanner and before detecting a syntax error.  Thus, state merging
       (from LALR or IELR) and default reductions corrupt the expected
       token list.  However, the list is correct for canonical LR with
       one exception: it will still contain any token that will not be
       accepted due to an error action in a later state.
  */
  if (yytoken != YYEMPTY)
    {
      int yyn = yypact[*yyssp];
      yyarg[yycount++] = yytname[yytoken];
      if (!yypact_value_is_default (yyn))
        {
          /* Start YYX at -YYN if negative to avoid negative indexes in
             YYCHECK.  In other words, skip the first -YYN actions for
             this state because they are default actions.  */
          int yyxbegin = yyn < 0 ? -yyn : 0;
          /* Stay within bounds of both yycheck and yytname.  */
          int yychecklim = YYLAST - yyn + 1;
          int yyxend = yychecklim < YYNTOKENS ? yychecklim : YYNTOKENS;
          int yyx;

          for (yyx = yyxbegin; yyx < yyxend; ++yyx)
            if (yycheck[yyx + yyn] == yyx && yyx != YYTERROR
                && !yytable_value_is_error (yytable[yyx + yyn]))
              {
                if (yycount == YYERROR_VERBOSE_ARGS_MAXIMUM)
                  {
                    yycount = 1;
                    yysize = yysize0;
                    break;
                  }
                yyarg[yycount++] = yytname[yyx];
                {
                  YYSIZE_T yysize1 = yysize + yytnamerr (YY_NULL, yytname[yyx]);
                  if (! (yysize <= yysize1
                         && yysize1 <= YYSTACK_ALLOC_MAXIMUM))
                    return 2;
                  yysize = yysize1;
                }
              }
        }
    }

  switch (yycount)
    {
# define YYCASE_(N, S)                      \
      case N:                               \
        yyformat = S;                       \
      break
      YYCASE_(0, YY_("syntax error"));
      YYCASE_(1, YY_("syntax error, unexpected %s"));
      YYCASE_(2, YY_("syntax error, unexpected %s, expecting %s"));
      YYCASE_(3, YY_("syntax error, unexpected %s, expecting %s or %s"));
      YYCASE_(4, YY_("syntax error, unexpected %s, expecting %s or %s or %s"));
      YYCASE_(5, YY_("syntax error, unexpected %s, expecting %s or %s or %s or %s"));
# undef YYCASE_
    }

  {
    YYSIZE_T yysize1 = yysize + yystrlen (yyformat);
    if (! (yysize <= yysize1 && yysize1 <= YYSTACK_ALLOC_MAXIMUM))
      return 2;
    yysize = yysize1;
  }

  if (*yymsg_alloc < yysize)
    {
      *yymsg_alloc = 2 * yysize;
      if (! (yysize <= *yymsg_alloc
             && *yymsg_alloc <= YYSTACK_ALLOC_MAXIMUM))
        *yymsg_alloc = YYSTACK_ALLOC_MAXIMUM;
      return 1;
    }

  /* Avoid sprintf, as that infringes on the user's name space.
     Don't have undefined behavior even if the translation
     produced a string with the wrong number of "%s"s.  */
  {
    char *yyp = *yymsg;
    int yyi = 0;
    while ((*yyp = *yyformat) != '\0')
      if (*yyp == '%' && yyformat[1] == 's' && yyi < yycount)
        {
          yyp += yytnamerr (yyp, yyarg[yyi++]);
          yyformat += 2;
        }
      else
        {
          yyp++;
          yyformat++;
        }
  }
  return 0;
}
#endif /* YYERROR_VERBOSE */

/*-----------------------------------------------.
| Release the memory associated to this symbol.  |
`-----------------------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yydestruct (const char *yymsg, int yytype, YYSTYPE *yyvaluep)
#else
static void
yydestruct (yymsg, yytype, yyvaluep)
    const char *yymsg;
    int yytype;
    YYSTYPE *yyvaluep;
#endif
{
  YYUSE (yyvaluep);

  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yytype, yyvaluep, yylocationp);

  switch (yytype)
    {

      default:
        break;
    }
}




/* The lookahead symbol.  */
int yychar;


#ifndef YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_END
#endif
#ifndef YY_INITIAL_VALUE
# define YY_INITIAL_VALUE(Value) /* Nothing. */
#endif

/* The semantic value of the lookahead symbol.  */
YYSTYPE yylval YY_INITIAL_VALUE(yyval_default);

/* Number of syntax errors so far.  */
int yynerrs;


/*----------.
| yyparse.  |
`----------*/

#ifdef YYPARSE_PARAM
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void *YYPARSE_PARAM)
#else
int
yyparse (YYPARSE_PARAM)
    void *YYPARSE_PARAM;
#endif
#else /* ! YYPARSE_PARAM */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void)
#else
int
yyparse ()

#endif
#endif
{
    int yystate;
    /* Number of tokens to shift before error messages enabled.  */
    int yyerrstatus;

    /* The stacks and their tools:
       `yyss': related to states.
       `yyvs': related to semantic values.

       Refer to the stacks through separate pointers, to allow yyoverflow
       to reallocate them elsewhere.  */

    /* The state stack.  */
    yytype_int16 yyssa[YYINITDEPTH];
    yytype_int16 *yyss;
    yytype_int16 *yyssp;

    /* The semantic value stack.  */
    YYSTYPE yyvsa[YYINITDEPTH];
    YYSTYPE *yyvs;
    YYSTYPE *yyvsp;

    YYSIZE_T yystacksize;

  int yyn;
  int yyresult;
  /* Lookahead token as an internal (translated) token number.  */
  int yytoken = 0;
  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;

#if YYERROR_VERBOSE
  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYSIZE_T yymsg_alloc = sizeof yymsgbuf;
#endif

#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N))

  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  yyssp = yyss = yyssa;
  yyvsp = yyvs = yyvsa;
  yystacksize = YYINITDEPTH;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yystate = 0;
  yyerrstatus = 0;
  yynerrs = 0;
  yychar = YYEMPTY; /* Cause a token to be read.  */
  goto yysetstate;

/*------------------------------------------------------------.
| yynewstate -- Push a new state, which is found in yystate.  |
`------------------------------------------------------------*/
 yynewstate:
  /* In all cases, when you get here, the value and location stacks
     have just been pushed.  So pushing a state here evens the stacks.  */
  yyssp++;

 yysetstate:
  *yyssp = yystate;

  if (yyss + yystacksize - 1 <= yyssp)
    {
      /* Get the current used size of the three stacks, in elements.  */
      YYSIZE_T yysize = yyssp - yyss + 1;

#ifdef yyoverflow
      {
	/* Give user a chance to reallocate the stack.  Use copies of
	   these so that the &'s don't force the real ones into
	   memory.  */
	YYSTYPE *yyvs1 = yyvs;
	yytype_int16 *yyss1 = yyss;

	/* Each stack pointer address is followed by the size of the
	   data in use in that stack, in bytes.  This used to be a
	   conditional around just the two extra args, but that might
	   be undefined if yyoverflow is a macro.  */
	yyoverflow (YY_("memory exhausted"),
		    &yyss1, yysize * sizeof (*yyssp),
		    &yyvs1, yysize * sizeof (*yyvsp),
		    &yystacksize);

	yyss = yyss1;
	yyvs = yyvs1;
      }
#else /* no yyoverflow */
# ifndef YYSTACK_RELOCATE
      goto yyexhaustedlab;
# else
      /* Extend the stack our own way.  */
      if (YYMAXDEPTH <= yystacksize)
	goto yyexhaustedlab;
      yystacksize *= 2;
      if (YYMAXDEPTH < yystacksize)
	yystacksize = YYMAXDEPTH;

      {
	yytype_int16 *yyss1 = yyss;
	union yyalloc *yyptr =
	  (union yyalloc *) YYSTACK_ALLOC (YYSTACK_BYTES (yystacksize));
	if (! yyptr)
	  goto yyexhaustedlab;
	YYSTACK_RELOCATE (yyss_alloc, yyss);
	YYSTACK_RELOCATE (yyvs_alloc, yyvs);
#  undef YYSTACK_RELOCATE
	if (yyss1 != yyssa)
	  YYSTACK_FREE (yyss1);
      }
# endif
#endif /* no yyoverflow */

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;

      YYDPRINTF ((stderr, "Stack size increased to %lu\n",
		  (unsigned long int) yystacksize));

      if (yyss + yystacksize - 1 <= yyssp)
	YYABORT;
    }

  YYDPRINTF ((stderr, "Entering state %d\n", yystate));

  if (yystate == YYFINAL)
    YYACCEPT;

  goto yybackup;

/*-----------.
| yybackup.  |
`-----------*/
yybackup:

  /* Do appropriate processing given the current state.  Read a
     lookahead token if we need one and don't already have one.  */

  /* First try to decide what to do without reference to lookahead token.  */
  yyn = yypact[yystate];
  if (yypact_value_is_default (yyn))
    goto yydefault;

  /* Not known => get a lookahead token if don't already have one.  */

  /* YYCHAR is either YYEMPTY or YYEOF or a valid lookahead symbol.  */
  if (yychar == YYEMPTY)
    {
      YYDPRINTF ((stderr, "Reading a token: "));
      yychar = YYLEX;
    }

  if (yychar <= YYEOF)
    {
      yychar = yytoken = YYEOF;
      YYDPRINTF ((stderr, "Now at end of input.\n"));
    }
  else
    {
      yytoken = YYTRANSLATE (yychar);
      YY_SYMBOL_PRINT ("Next token is", yytoken, &yylval, &yylloc);
    }

  /* If the proper action on seeing token YYTOKEN is to reduce or to
     detect an error, take that action.  */
  yyn += yytoken;
  if (yyn < 0 || YYLAST < yyn || yycheck[yyn] != yytoken)
    goto yydefault;
  yyn = yytable[yyn];
  if (yyn <= 0)
    {
      if (yytable_value_is_error (yyn))
        goto yyerrlab;
      yyn = -yyn;
      goto yyreduce;
    }

  /* Count tokens shifted since error; after three, turn off error
     status.  */
  if (yyerrstatus)
    yyerrstatus--;

  /* Shift the lookahead token.  */
  YY_SYMBOL_PRINT ("Shifting", yytoken, &yylval, &yylloc);

  /* Discard the shifted token.  */
  yychar = YYEMPTY;

  yystate = yyn;
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END

  goto yynewstate;


/*-----------------------------------------------------------.
| yydefault -- do the default action for the current state.  |
`-----------------------------------------------------------*/
yydefault:
  yyn = yydefact[yystate];
  if (yyn == 0)
    goto yyerrlab;
  goto yyreduce;


/*-----------------------------.
| yyreduce -- Do a reduction.  |
`-----------------------------*/
yyreduce:
  /* yyn is the number of a rule to reduce with.  */
  yylen = yyr2[yyn];

  /* If YYLEN is nonzero, implement the default value of the action:
     `$$ = $1'.

     Otherwise, the following line sets YYVAL to garbage.
     This behavior is undocumented and Bison
     users should not rely upon it.  Assigning to YYVAL
     unconditionally makes the parser a bit smaller, and it avoids a
     GCC warning that YYVAL may be used uninitialized.  */
  yyval = yyvsp[1-yylen];


  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
        case 4:
/* Line 1792 of yacc.c  */
#line 112 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(1) - (1)].m_pStatement));
	}
    break;

  case 5:
/* Line 1792 of yacc.c  */
#line 115 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(2) - (2)].m_pStatement));
	}
    break;

  case 6:
/* Line 1792 of yacc.c  */
#line 121 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 29:
/* Line 1792 of yacc.c  */
#line 152 "iris.y"
    {
		IrisStatement* pStatement = new IrisClassStatement((yyvsp[(2) - (6)].m_pIdentifier), (yyvsp[(3) - (6)].m_pExpression), (yyvsp[(4) - (6)].m_pExpressionList), (yyvsp[(5) - (6)].m_pExpressionList), (yyvsp[(6) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 30:
/* Line 1792 of yacc.c  */
#line 160 "iris.y"
    {
		(yyval.m_pExpression) = nullptr;
	}
    break;

  case 31:
/* Line 1792 of yacc.c  */
#line 163 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (2)].m_pExpression);
	}
    break;

  case 32:
/* Line 1792 of yacc.c  */
#line 166 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), false));
	}
    break;

  case 33:
/* Line 1792 of yacc.c  */
#line 172 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 34:
/* Line 1792 of yacc.c  */
#line 175 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 35:
/* Line 1792 of yacc.c  */
#line 181 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 36:
/* Line 1792 of yacc.c  */
#line 184 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 37:
/* Line 1792 of yacc.c  */
#line 190 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(1) - (1)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = pList;		
	}
    break;

  case 38:
/* Line 1792 of yacc.c  */
#line 195 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pList;
	}
    break;

  case 39:
/* Line 1792 of yacc.c  */
#line 200 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(3) - (3)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 40:
/* Line 1792 of yacc.c  */
#line 204 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);		
	}
    break;

  case 41:
/* Line 1792 of yacc.c  */
#line 212 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 42:
/* Line 1792 of yacc.c  */
#line 217 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 43:
/* Line 1792 of yacc.c  */
#line 226 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 44:
/* Line 1792 of yacc.c  */
#line 231 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(3) - (8)].m_pIdentifier), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 45:
/* Line 1792 of yacc.c  */
#line 240 "iris.y"
    {
		IrisStatement* pStatement = new IrisGSetterStatement((yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
 		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 46:
/* Line 1792 of yacc.c  */
#line 249 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::EveryOne, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 47:
/* Line 1792 of yacc.c  */
#line 254 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::EveryOne, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 48:
/* Line 1792 of yacc.c  */
#line 262 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Relative, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 49:
/* Line 1792 of yacc.c  */
#line 267 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Relative, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 50:
/* Line 1792 of yacc.c  */
#line 275 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Personal, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 51:
/* Line 1792 of yacc.c  */
#line 280 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Personal, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 52:
/* Line 1792 of yacc.c  */
#line 288 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 53:
/* Line 1792 of yacc.c  */
#line 297 "iris.y"
    {
		IrisStatement* pStatement = new IrisModuleStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), nullptr, (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 54:
/* Line 1792 of yacc.c  */
#line 306 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 55:
/* Line 1792 of yacc.c  */
#line 311 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (6)].m_pFunctionHeader), (yyvsp[(4) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pBlock), (yyvsp[(2) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 56:
/* Line 1792 of yacc.c  */
#line 319 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 57:
/* Line 1792 of yacc.c  */
#line 322 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 58:
/* Line 1792 of yacc.c  */
#line 325 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (8)].m_pIdentifier), (yyvsp[(4) - (8)].m_pIdentifierList), (yyvsp[(7) - (8)].m_pIdentifier), false);
	}
    break;

  case 59:
/* Line 1792 of yacc.c  */
#line 328 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (6)].m_pIdentifier), nullptr, (yyvsp[(5) - (6)].m_pIdentifier), false);
	}
    break;

  case 60:
/* Line 1792 of yacc.c  */
#line 331 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (6)].m_pIdentifier), nullptr, nullptr, true);
	}
    break;

  case 61:
/* Line 1792 of yacc.c  */
#line 334 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (7)].m_pIdentifier), (yyvsp[(6) - (7)].m_pIdentifierList), nullptr, true);
	}
    break;

  case 62:
/* Line 1792 of yacc.c  */
#line 337 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (10)].m_pIdentifier), (yyvsp[(6) - (10)].m_pIdentifierList), (yyvsp[(9) - (10)].m_pIdentifier), true);
	}
    break;

  case 63:
/* Line 1792 of yacc.c  */
#line 340 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (8)].m_pIdentifier), nullptr, (yyvsp[(7) - (8)].m_pIdentifier), true);
	}
    break;

  case 64:
/* Line 1792 of yacc.c  */
#line 346 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 65:
/* Line 1792 of yacc.c  */
#line 354 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 66:
/* Line 1792 of yacc.c  */
#line 357 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 67:
/* Line 1792 of yacc.c  */
#line 368 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "[]");
	}
    break;

  case 68:
/* Line 1792 of yacc.c  */
#line 371 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "[]=");
	}
    break;

  case 69:
/* Line 1792 of yacc.c  */
#line 374 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "+");
	}
    break;

  case 70:
/* Line 1792 of yacc.c  */
#line 377 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "||");
	}
    break;

  case 71:
/* Line 1792 of yacc.c  */
#line 380 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "!");
	}
    break;

  case 72:
/* Line 1792 of yacc.c  */
#line 383 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "==");
	}
    break;

  case 73:
/* Line 1792 of yacc.c  */
#line 386 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "!=");
	}
    break;

  case 74:
/* Line 1792 of yacc.c  */
#line 389 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, ">");
	}
    break;

  case 75:
/* Line 1792 of yacc.c  */
#line 392 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, ">=");
	}
    break;

  case 76:
/* Line 1792 of yacc.c  */
#line 395 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "<");
	}
    break;

  case 77:
/* Line 1792 of yacc.c  */
#line 398 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "<=");
	}
    break;

  case 78:
/* Line 1792 of yacc.c  */
#line 401 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "+");
	}
    break;

  case 79:
/* Line 1792 of yacc.c  */
#line 404 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "-");
	}
    break;

  case 80:
/* Line 1792 of yacc.c  */
#line 407 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "*");
	}
    break;

  case 81:
/* Line 1792 of yacc.c  */
#line 410 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "/");
	}
    break;

  case 82:
/* Line 1792 of yacc.c  */
#line 413 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "%");
	}
    break;

  case 83:
/* Line 1792 of yacc.c  */
#line 416 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "**");
	}
    break;

  case 84:
/* Line 1792 of yacc.c  */
#line 419 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "&");
	}
    break;

  case 85:
/* Line 1792 of yacc.c  */
#line 422 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "|");
	}
    break;

  case 86:
/* Line 1792 of yacc.c  */
#line 425 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "^");
	}
    break;

  case 87:
/* Line 1792 of yacc.c  */
#line 428 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "!");
	}
    break;

  case 88:
/* Line 1792 of yacc.c  */
#line 431 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, ">>");
	}
    break;

  case 89:
/* Line 1792 of yacc.c  */
#line 434 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "<<");
	}
    break;

  case 90:
/* Line 1792 of yacc.c  */
#line 437 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, ">>>");
	}
    break;

  case 91:
/* Line 1792 of yacc.c  */
#line 440 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifierType::GlobalVariable, "<<<");
	}
    break;

  case 92:
/* Line 1792 of yacc.c  */
#line 447 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pConditionIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pConditionIf);
	}
    break;

  case 93:
/* Line 1792 of yacc.c  */
#line 451 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pLoopIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pLoopIf);
	}
    break;

  case 94:
/* Line 1792 of yacc.c  */
#line 458 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock), nullptr, nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 95:
/* Line 1792 of yacc.c  */
#line 462 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (7)].m_pExpression), (yyvsp[(5) - (7)].m_pBlock), nullptr, (yyvsp[(7) - (7)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 96:
/* Line 1792 of yacc.c  */
#line 467 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pElseIfList), nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 97:
/* Line 1792 of yacc.c  */
#line 471 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pBlock), (yyvsp[(6) - (8)].m_pElseIfList), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 98:
/* Line 1792 of yacc.c  */
#line 478 "iris.y"
    {
		IrisList<IrisElseIf*>* pElseIfList = new IrisList<IrisElseIf*>();
		pElseIfList->Add((yyvsp[(1) - (1)].m_pElseIf));
		(yyval.m_pElseIfList) = pElseIfList;
	}
    break;

  case 99:
/* Line 1792 of yacc.c  */
#line 483 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pElseIfList)->Add((yyvsp[(2) - (2)].m_pElseIf));
		(yyval.m_pElseIfList) = (yyvsp[(1) - (2)].m_pElseIfList);
	}
    break;

  case 100:
/* Line 1792 of yacc.c  */
#line 490 "iris.y"
    {
		IrisElseIf* pElseIf = new IrisElseIf((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pElseIf) = pElseIf;
	}
    break;

  case 101:
/* Line 1792 of yacc.c  */
#line 497 "iris.y"
    {
		IrisLoopIfStatement* pLoopIf = new IrisLoopIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pExpression), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pLoopIf) = pLoopIf;
	}
    break;

  case 102:
/* Line 1792 of yacc.c  */
#line 504 "iris.y"
    {
		(yyval.m_pIdentifier) = nullptr;
	}
    break;

  case 103:
/* Line 1792 of yacc.c  */
#line 507 "iris.y"
    {
		(yyval.m_pIdentifier) = (yyvsp[(2) - (2)].m_pIdentifier);
	}
    break;

  case 104:
/* Line 1792 of yacc.c  */
#line 514 "iris.y"
    {
		IrisStatement* pStatement = new IrisSwitchStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pSwitchBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 105:
/* Line 1792 of yacc.c  */
#line 522 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (3)].m_pWhenList), nullptr);
		(yyval.m_pSwitchBlock) = pSwitchBlock;
	}
    break;

  case 106:
/* Line 1792 of yacc.c  */
#line 526 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (5)].m_pWhenList), (yyvsp[(4) - (5)].m_pBlock));
		(yyval.m_pSwitchBlock) = pSwitchBlock;		
	}
    break;

  case 107:
/* Line 1792 of yacc.c  */
#line 533 "iris.y"
    {
		IrisList<IrisWhen*>* pWhenList = new IrisList<IrisWhen*>();
		pWhenList->Add((yyvsp[(1) - (1)].m_pWhen));
		(yyval.m_pWhenList) = pWhenList;
	}
    break;

  case 108:
/* Line 1792 of yacc.c  */
#line 538 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pWhenList)->Add((yyvsp[(2) - (2)].m_pWhen));
		(yyval.m_pWhenList) = (yyvsp[(1) - (2)].m_pWhenList);
	}
    break;

  case 109:
/* Line 1792 of yacc.c  */
#line 545 "iris.y"
    {
		IrisWhen* pWhen = new IrisWhen((yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pWhen) = pWhen;
	}
    break;

  case 110:
/* Line 1792 of yacc.c  */
#line 553 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(5) - (7)].m_pExpression), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 111:
/* Line 1792 of yacc.c  */
#line 558 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(4) - (11)].m_pIdentifier), (yyvsp[(6) - (11)].m_pIdentifier), (yyvsp[(9) - (11)].m_pExpression), (yyvsp[(11) - (11)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 112:
/* Line 1792 of yacc.c  */
#line 567 "iris.y"
    {
		IrisStatement* pStatement = new IrisOrderStatement((yyvsp[(2) - (8)].m_pBlock), (yyvsp[(5) - (8)].m_pIdentifier), (yyvsp[(7) - (8)].m_pBlock), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 113:
/* Line 1792 of yacc.c  */
#line 575 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 114:
/* Line 1792 of yacc.c  */
#line 578 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(3) - (4)].m_pStatementList));
	}
    break;

  case 115:
/* Line 1792 of yacc.c  */
#line 586 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 116:
/* Line 1792 of yacc.c  */
#line 591 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement((yyvsp[(3) - (3)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 117:
/* Line 1792 of yacc.c  */
#line 600 "iris.y"
    {
		IrisStatement* pStatement = new IrisBreakStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 118:
/* Line 1792 of yacc.c  */
#line 608 "iris.y"
    {
		IrisStatement* pStatement = new IrisContinueStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 119:
/* Line 1792 of yacc.c  */
#line 617 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 120:
/* Line 1792 of yacc.c  */
#line 622 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (6)].m_pIdentifier), (yyvsp[(5) - (6)].m_pIdentifierList), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 121:
/* Line 1792 of yacc.c  */
#line 627 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (9)].m_pIdentifier), (yyvsp[(5) - (9)].m_pIdentifierList), (yyvsp[(8) - (9)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 122:
/* Line 1792 of yacc.c  */
#line 632 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 123:
/* Line 1792 of yacc.c  */
#line 641 "iris.y"
    {
		IrisStatement* pStatement = new IrisGroanStatement((yyvsp[(4) - (5)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 124:
/* Line 1792 of yacc.c  */
#line 650 "iris.y"
    {
		IrisStatement* pStatement = new IrisBlockStatement();
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 125:
/* Line 1792 of yacc.c  */
#line 659 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 126:
/* Line 1792 of yacc.c  */
#line 664 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement((yyvsp[(4) - (5)].m_pExpressionList));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;	
	}
    break;

  case 127:
/* Line 1792 of yacc.c  */
#line 672 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock(nullptr);
		(yyval.m_pBlock) = pBlock;
	}
    break;

  case 128:
/* Line 1792 of yacc.c  */
#line 676 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
		(yyval.m_pBlock) = pBlock;		
	}
    break;

  case 129:
/* Line 1792 of yacc.c  */
#line 682 "iris.y"
    {
		IrisList<IrisStatement*>* pStatementList = new IrisList<IrisStatement*>();
		pStatementList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pStatementList;
	}
    break;

  case 130:
/* Line 1792 of yacc.c  */
#line 687 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 132:
/* Line 1792 of yacc.c  */
#line 696 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression((yyvsp[(2) - (3)].m_eExpressionType), (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 133:
/* Line 1792 of yacc.c  */
#line 703 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::Assign;
	}
    break;

  case 134:
/* Line 1792 of yacc.c  */
#line 706 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignAdd;
	}
    break;

  case 135:
/* Line 1792 of yacc.c  */
#line 709 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignSub;
	}
    break;

  case 136:
/* Line 1792 of yacc.c  */
#line 712 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMul;
	}
    break;

  case 137:
/* Line 1792 of yacc.c  */
#line 715 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignDiv;
	}
    break;

  case 138:
/* Line 1792 of yacc.c  */
#line 718 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMod;
	}
    break;

  case 139:
/* Line 1792 of yacc.c  */
#line 721 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignPower;
	}
    break;

  case 140:
/* Line 1792 of yacc.c  */
#line 724 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitAnd;
	}
    break;

  case 141:
/* Line 1792 of yacc.c  */
#line 727 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitOr;
	}
    break;

  case 142:
/* Line 1792 of yacc.c  */
#line 730 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitXor;
	}
    break;

  case 143:
/* Line 1792 of yacc.c  */
#line 733 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShr;
	}
    break;

  case 144:
/* Line 1792 of yacc.c  */
#line 736 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShl;
	}
    break;

  case 145:
/* Line 1792 of yacc.c  */
#line 739 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSar;
	}
    break;

  case 146:
/* Line 1792 of yacc.c  */
#line 742 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSal;
	}
    break;

  case 148:
/* Line 1792 of yacc.c  */
#line 749 "iris.y"
    {
		(yyval.m_pExpression) =  new IrisBinaryExpression(IrisBinaryExpressionType::LogicOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 150:
/* Line 1792 of yacc.c  */
#line 757 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 152:
/* Line 1792 of yacc.c  */
#line 765 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 154:
/* Line 1792 of yacc.c  */
#line 773 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitXor, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 156:
/* Line 1792 of yacc.c  */
#line 781 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 158:
/* Line 1792 of yacc.c  */
#line 789 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Equal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 159:
/* Line 1792 of yacc.c  */
#line 793 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::NotEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 161:
/* Line 1792 of yacc.c  */
#line 801 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 162:
/* Line 1792 of yacc.c  */
#line 805 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 163:
/* Line 1792 of yacc.c  */
#line 809 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 164:
/* Line 1792 of yacc.c  */
#line 813 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 166:
/* Line 1792 of yacc.c  */
#line 821 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 167:
/* Line 1792 of yacc.c  */
#line 825 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShl, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 168:
/* Line 1792 of yacc.c  */
#line 829 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSar, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 169:
/* Line 1792 of yacc.c  */
#line 833 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 171:
/* Line 1792 of yacc.c  */
#line 841 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Add, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 172:
/* Line 1792 of yacc.c  */
#line 845 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Sub, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 174:
/* Line 1792 of yacc.c  */
#line 853 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mul, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 175:
/* Line 1792 of yacc.c  */
#line 857 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Div, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 176:
/* Line 1792 of yacc.c  */
#line 861 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mod, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 178:
/* Line 1792 of yacc.c  */
#line 869 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Power, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 180:
/* Line 1792 of yacc.c  */
#line 877 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::LogicNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 181:
/* Line 1792 of yacc.c  */
#line 881 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::BitNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 182:
/* Line 1792 of yacc.c  */
#line 885 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Minus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 183:
/* Line 1792 of yacc.c  */
#line 889 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Plus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 185:
/* Line 1792 of yacc.c  */
#line 897 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIndexExpression((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 186:
/* Line 1792 of yacc.c  */
#line 901 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (6)].m_pExpression), (yyvsp[(3) - (6)].m_pIdentifier), nullptr, (yyvsp[(6) - (6)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 187:
/* Line 1792 of yacc.c  */
#line 905 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (7)].m_pExpression), (yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(5) - (7)].m_pExpressionList), (yyvsp[(7) - (7)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 188:
/* Line 1792 of yacc.c  */
#line 909 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (4)].m_pIdentifier), nullptr, (yyvsp[(4) - (4)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 189:
/* Line 1792 of yacc.c  */
#line 913 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (5)].m_pIdentifier), (yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 190:
/* Line 1792 of yacc.c  */
#line 917 "iris.y"
    {
		(yyval.m_pExpression) = new IrisMemberExpression((yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 191:
/* Line 1792 of yacc.c  */
#line 924 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 192:
/* Line 1792 of yacc.c  */
#line 927 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 193:
/* Line 1792 of yacc.c  */
#line 930 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral(nullptr, nullptr, (yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 194:
/* Line 1792 of yacc.c  */
#line 933 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral((yyvsp[(5) - (9)].m_pIdentifierList), nullptr, (yyvsp[(8) - (9)].m_pStatementList));
	}
    break;

  case 195:
/* Line 1792 of yacc.c  */
#line 936 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral((yyvsp[(5) - (12)].m_pIdentifierList), (yyvsp[(8) - (12)].m_pIdentifier), (yyvsp[(11) - (12)].m_pStatementList));
	}
    break;

  case 196:
/* Line 1792 of yacc.c  */
#line 939 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral(nullptr, (yyvsp[(6) - (10)].m_pIdentifier), (yyvsp[(9) - (10)].m_pStatementList));
	}
    break;

  case 198:
/* Line 1792 of yacc.c  */
#line 946 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (3)].m_pExpression);
	}
    break;

  case 199:
/* Line 1792 of yacc.c  */
#line 949 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIdentifierExpression((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 200:
/* Line 1792 of yacc.c  */
#line 953 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 201:
/* Line 1792 of yacc.c  */
#line 957 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 202:
/* Line 1792 of yacc.c  */
#line 961 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 203:
/* Line 1792 of yacc.c  */
#line 965 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::True);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 204:
/* Line 1792 of yacc.c  */
#line 969 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::False);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 205:
/* Line 1792 of yacc.c  */
#line 973 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::Nil);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 206:
/* Line 1792 of yacc.c  */
#line 977 "iris.y"
    {
		(yyval.m_pExpression) = new IrisSelfExpression();
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 207:
/* Line 1792 of yacc.c  */
#line 981 "iris.y"
    {
		(yyval.m_pExpression) = new IrisCastExpression();
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 211:
/* Line 1792 of yacc.c  */
#line 991 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(1) - (2)].m_pIdentifierList), (yyvsp[(2) - (2)].m_pIdentifier), false));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 212:
/* Line 1792 of yacc.c  */
#line 995 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(2) - (3)].m_pIdentifierList), (yyvsp[(3) - (3)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 213:
/* Line 1792 of yacc.c  */
#line 999 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 214:
/* Line 1792 of yacc.c  */
#line 1006 "iris.y"
    {
		(yyval.m_pExpression) = new IrisArrayExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 215:
/* Line 1792 of yacc.c  */
#line 1010 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (3)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 216:
/* Line 1792 of yacc.c  */
#line 1014 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (4)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 217:
/* Line 1792 of yacc.c  */
#line 1022 "iris.y"
    {
		IrisList<IrisIdentifier*>* pFieldIdentifier = new IrisList<IrisIdentifier*>();
		pFieldIdentifier->Add((yyvsp[(1) - (2)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pFieldIdentifier;
	}
    break;

  case 218:
/* Line 1792 of yacc.c  */
#line 1027 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(2) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 219:
/* Line 1792 of yacc.c  */
#line 1035 "iris.y"
    {
		IrisList<IrisIdentifier*>* pList = new IrisList<IrisIdentifier*>();
		pList->Add((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pList;
	}
    break;

  case 220:
/* Line 1792 of yacc.c  */
#line 1040 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 221:
/* Line 1792 of yacc.c  */
#line 1047 "iris.y"
    {
		IrisList<IrisExpression*>* pExpressionList = new IrisList<IrisExpression*>();
		pExpressionList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pExpressionList;
	}
    break;

  case 222:
/* Line 1792 of yacc.c  */
#line 1052 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 223:
/* Line 1792 of yacc.c  */
#line 1059 "iris.y"
    {
		(yyval.m_pExpression) = new IrisHashExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 224:
/* Line 1792 of yacc.c  */
#line 1063 "iris.y"
    {
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (3)].m_pHashPairList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 225:
/* Line 1792 of yacc.c  */
#line 1067 "iris.y"
    {
		(yyvsp[(2) - (6)].m_pHashPairList)->Add(new IrisHashPair((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pExpression)));
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (6)].m_pHashPairList));
    }
    break;

  case 226:
/* Line 1792 of yacc.c  */
#line 1074 "iris.y"
    {
		IrisList<IrisHashPair*>* pHashPairList = new IrisList<IrisHashPair*>();
		pHashPairList->Add((yyvsp[(1) - (1)].m_pHashPair));
		(yyval.m_pHashPairList) = pHashPairList;
	}
    break;

  case 227:
/* Line 1792 of yacc.c  */
#line 1079 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pHashPairList)->Add((yyvsp[(2) - (2)].m_pHashPair));
		(yyval.m_pHashPairList) = (yyvsp[(1) - (2)].m_pHashPairList);
	}
    break;

  case 228:
/* Line 1792 of yacc.c  */
#line 1085 "iris.y"
    {
		(yyval.m_pHashPair) = new IrisHashPair((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
	}
    break;

  case 229:
/* Line 1792 of yacc.c  */
#line 1090 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 230:
/* Line 1792 of yacc.c  */
#line 1094 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
	}
    break;

  case 231:
/* Line 1792 of yacc.c  */
#line 1098 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());		
	}
    break;

  case 232:
/* Line 1792 of yacc.c  */
#line 1102 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());			
	}
    break;


/* Line 1792 of yacc.c  */
#line 3788 "y.tab.c"
      default: break;
    }
  /* User semantic actions sometimes alter yychar, and that requires
     that yytoken be updated with the new translation.  We take the
     approach of translating immediately before every use of yytoken.
     One alternative is translating here after every semantic action,
     but that translation would be missed if the semantic action invokes
     YYABORT, YYACCEPT, or YYERROR immediately after altering yychar or
     if it invokes YYBACKUP.  In the case of YYABORT or YYACCEPT, an
     incorrect destructor might then be invoked immediately.  In the
     case of YYERROR or YYBACKUP, subsequent parser actions might lead
     to an incorrect destructor call or verbose syntax error message
     before the lookahead is translated.  */
  YY_SYMBOL_PRINT ("-> $$ =", yyr1[yyn], &yyval, &yyloc);

  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);

  *++yyvsp = yyval;

  /* Now `shift' the result of the reduction.  Determine what state
     that goes to, based on the state we popped back to and the rule
     number reduced by.  */

  yyn = yyr1[yyn];

  yystate = yypgoto[yyn - YYNTOKENS] + *yyssp;
  if (0 <= yystate && yystate <= YYLAST && yycheck[yystate] == *yyssp)
    yystate = yytable[yystate];
  else
    yystate = yydefgoto[yyn - YYNTOKENS];

  goto yynewstate;


/*------------------------------------.
| yyerrlab -- here on detecting error |
`------------------------------------*/
yyerrlab:
  /* Make sure we have latest lookahead translation.  See comments at
     user semantic actions for why this is necessary.  */
  yytoken = yychar == YYEMPTY ? YYEMPTY : YYTRANSLATE (yychar);

  /* If not already recovering from an error, report this error.  */
  if (!yyerrstatus)
    {
      ++yynerrs;
#if ! YYERROR_VERBOSE
      yyerror (YY_("syntax error"));
#else
# define YYSYNTAX_ERROR yysyntax_error (&yymsg_alloc, &yymsg, \
                                        yyssp, yytoken)
      {
        char const *yymsgp = YY_("syntax error");
        int yysyntax_error_status;
        yysyntax_error_status = YYSYNTAX_ERROR;
        if (yysyntax_error_status == 0)
          yymsgp = yymsg;
        else if (yysyntax_error_status == 1)
          {
            if (yymsg != yymsgbuf)
              YYSTACK_FREE (yymsg);
            yymsg = (char *) YYSTACK_ALLOC (yymsg_alloc);
            if (!yymsg)
              {
                yymsg = yymsgbuf;
                yymsg_alloc = sizeof yymsgbuf;
                yysyntax_error_status = 2;
              }
            else
              {
                yysyntax_error_status = YYSYNTAX_ERROR;
                yymsgp = yymsg;
              }
          }
        yyerror (yymsgp);
        if (yysyntax_error_status == 2)
          goto yyexhaustedlab;
      }
# undef YYSYNTAX_ERROR
#endif
    }



  if (yyerrstatus == 3)
    {
      /* If just tried and failed to reuse lookahead token after an
	 error, discard it.  */

      if (yychar <= YYEOF)
	{
	  /* Return failure if at end of input.  */
	  if (yychar == YYEOF)
	    YYABORT;
	}
      else
	{
	  yydestruct ("Error: discarding",
		      yytoken, &yylval);
	  yychar = YYEMPTY;
	}
    }

  /* Else will try to reuse lookahead token after shifting the error
     token.  */
  goto yyerrlab1;


/*---------------------------------------------------.
| yyerrorlab -- error raised explicitly by YYERROR.  |
`---------------------------------------------------*/
yyerrorlab:

  /* Pacify compilers like GCC when the user code never invokes
     YYERROR and the label yyerrorlab therefore never appears in user
     code.  */
  if (/*CONSTCOND*/ 0)
     goto yyerrorlab;

  /* Do not reclaim the symbols of the rule which action triggered
     this YYERROR.  */
  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);
  yystate = *yyssp;
  goto yyerrlab1;


/*-------------------------------------------------------------.
| yyerrlab1 -- common code for both syntax error and YYERROR.  |
`-------------------------------------------------------------*/
yyerrlab1:
  yyerrstatus = 3;	/* Each real token shifted decrements this.  */

  for (;;)
    {
      yyn = yypact[yystate];
      if (!yypact_value_is_default (yyn))
	{
	  yyn += YYTERROR;
	  if (0 <= yyn && yyn <= YYLAST && yycheck[yyn] == YYTERROR)
	    {
	      yyn = yytable[yyn];
	      if (0 < yyn)
		break;
	    }
	}

      /* Pop the current state because it cannot handle the error token.  */
      if (yyssp == yyss)
	YYABORT;


      yydestruct ("Error: popping",
		  yystos[yystate], yyvsp);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END


  /* Shift the error token.  */
  YY_SYMBOL_PRINT ("Shifting", yystos[yyn], yyvsp, yylsp);

  yystate = yyn;
  goto yynewstate;


/*-------------------------------------.
| yyacceptlab -- YYACCEPT comes here.  |
`-------------------------------------*/
yyacceptlab:
  yyresult = 0;
  goto yyreturn;

/*-----------------------------------.
| yyabortlab -- YYABORT comes here.  |
`-----------------------------------*/
yyabortlab:
  yyresult = 1;
  goto yyreturn;

#if !defined yyoverflow || YYERROR_VERBOSE
/*-------------------------------------------------.
| yyexhaustedlab -- memory exhaustion comes here.  |
`-------------------------------------------------*/
yyexhaustedlab:
  yyerror (YY_("memory exhausted"));
  yyresult = 2;
  /* Fall through.  */
#endif

yyreturn:
  if (yychar != YYEMPTY)
    {
      /* Make sure we have latest lookahead translation.  See comments at
         user semantic actions for why this is necessary.  */
      yytoken = YYTRANSLATE (yychar);
      yydestruct ("Cleanup: discarding lookahead",
                  yytoken, &yylval);
    }
  /* Do not reclaim the symbols of the rule which action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
		  yystos[*yyssp], yyvsp);
      YYPOPSTACK (1);
    }
#ifndef yyoverflow
  if (yyss != yyssa)
    YYSTACK_FREE (yyss);
#endif
#if YYERROR_VERBOSE
  if (yymsg != yymsgbuf)
    YYSTACK_FREE (yymsg);
#endif
  /* Make sure YYID is used.  */
  return YYID (yyresult);
}


/* Line 2055 of yacc.c  */
#line 1106 "iris.y"
