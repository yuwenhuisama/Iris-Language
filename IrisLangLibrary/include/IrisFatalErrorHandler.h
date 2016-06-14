#ifndef _H_IRISFATLEERRORHANDLE_
#define _H_IRISFATLEERRORHANDLE_

#include <string>
using namespace std;

class IrisFatalErrorHandler
{
public:

	typedef void(*FatalErrorMessageFunction)(const string& strMessage);

	enum class FatalErrorType {
		SyntaxIrregular = 0,
		IdenfierTypeIrregular,
		NoMethodIrregular,
		IrregularNotDealedIrregular,
		SourceObjectIrregular,
		IdentifierNotFoundIrregular,
		ReturnIrregular,
		LeftValueIrregular,
		SelfPointerIrregular,
		ConstanceReassignIrregular,
		ConstanceDeclareIrregular,
		ClassVariableDeclareIrregular,
		MethodAuthorityIrregular,
		ClassNotCompleteIrregular,
		ParameterNotFitIrregular,
		NoMethodCanSuperIrregular,
		FieldCannotRoutedIrregular,
		ContinueIrregular,
		BreakIrregular,
		UnkownIrregular,
	};

private:
	static IrisFatalErrorHandler* sm_pInstance;

private:
	FatalErrorMessageFunction s_pfMessageFunction;

public:
	static IrisFatalErrorHandler* CurrentFatalHandler();
	static void SetCurrentFatalHandler(IrisFatalErrorHandler* pHandler);

public:
	void SetFatalErrorMessageFuncton(FatalErrorMessageFunction pfFunction);
	void ShowFatalErrorMessage(FatalErrorType eType, size_t nLineNumber, size_t nBelongingFileIndex, const string & strFatalErrorMessage);
	void ShowFatalErrorDirectly(const string& strFatalErrorMessage);

private:
	IrisFatalErrorHandler();
public:
	~IrisFatalErrorHandler();
};

#endif