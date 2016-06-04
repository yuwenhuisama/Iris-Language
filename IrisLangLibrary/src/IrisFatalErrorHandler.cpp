#include "IrisFatalErrorHandler.h"
#include "IrisCompiler.h"
#include "IrisInterpreter.h"
#include <sstream>

IrisFatalErrorHandler* IrisFatalErrorHandler::sm_pInstance = nullptr;

IrisFatalErrorHandler * IrisFatalErrorHandler::CurrentFatalHandler()
{
	if (!sm_pInstance) {
		sm_pInstance = new IrisFatalErrorHandler();
	}
	return sm_pInstance;
}

void IrisFatalErrorHandler::SetCurrentFatalHandler(IrisFatalErrorHandler* pHandler)
{
	sm_pInstance = pHandler;
}

void IrisFatalErrorHandler::SetFatalErrorMessageFuncton(FatalErrorMessageFunction pfFunction)
{
	s_pfMessageFunction = pfFunction;
}

void IrisFatalErrorHandler::ShowFatalErrorMessage(FatalErrorType eType, int nLineNumber, int nBelongingFileIndex, const string & strFatalErrorMessage)
{

	IrisInterpreter::CurrentInterpreter()->HappenFatalError();

	string strIrregularTitle = "<Irregular : ";
	string strMessage = "\n  Irregular-happened Report : Oh! Master, a FATAL ERROR has happened and Iris is not clever and dosen't kown how to deal with it. Could you please cheak it yourself? \n";
	string strIrregularMessage = ">The Irregular name is ";
	stringstream ssStream;
	ssStream << nLineNumber;
	string strLinenoMessage = "";
	if (nLineNumber > 0) {
		strLinenoMessage = ">and happened at line " + ssStream.str() + " file " + IrisCompiler::CurrentCompiler()->GetFileName(nBelongingFileIndex).GetSTLString() +".\n";
	}

	switch (eType)
	{
	case FatalErrorType::SyntaxIrregular:
		strMessage = strIrregularTitle + "SyntaxIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SyntaxIrregular," + "\n" + strLinenoMessage;		
		break;
	case FatalErrorType::NoMethodIrregular:
		strMessage = strIrregularTitle + "NoMethodIrregular>" + strMessage;
		strMessage += strIrregularMessage + "NoMethodIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IdenfierTypeIrregular:
		strMessage = strIrregularTitle + "IdenfierTypeIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IdenfierTypeIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IrregularNotDealedIrregular:
		strMessage = strIrregularTitle + "IrregularNotDealedIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IrregularNotDealedIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::SourceObjectIrregular:
		strMessage = strIrregularTitle + "SourceObjectIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SourceObjectIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IdentifierNotFoundIrregular:
		strMessage = strIrregularTitle + "IdentifierNotFoundIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IdentifierNotFoundIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ReturnIrregular:
		strMessage = strIrregularTitle + "ReturnIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ReturnIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::LeftValueIrregular:
		strMessage = strIrregularTitle + "LeftValueIrregular>" + strMessage;
		strMessage += strIrregularMessage + "LeftValueIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::SelfPointerIrregular:
		strMessage = strIrregularTitle + "SelfPointerIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SelfPointerIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ConstanceReassignIrregular:
		strMessage = strIrregularTitle + "ConstanceReassignIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ConstanceReassignIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ConstanceDeclareIrregular:
		strMessage = strIrregularTitle + "ConstanceDeclareIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ConstanceDeclareIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ClassVariableDeclareIrregular:
		strMessage = strIrregularTitle + "ClassVariableDeclareIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ClassVariableDeclareIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::MethodAuthorityIrregular:
		strMessage = strIrregularTitle + "MethodAuthorityIrregular>" + strMessage;
		strMessage += strIrregularMessage + "MethodAuthorityIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ClassNotCompleteIrregular:
		strMessage = strIrregularTitle + "ClassNotCompleteIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ClassNotCompleteIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ParameterNotFitIrregular:
		strMessage = strIrregularTitle + "ParameterNotFitIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ParameterNotFitIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::NoMethodCanSuperIrregular:
		strMessage = strIrregularTitle + "NoMethodCanSuperIrregular>" + strMessage;
		strMessage += strIrregularMessage + "NoMethodCanSuperIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::UnkownIrregular:
		strMessage = strIrregularTitle + "UnkownIrregular>" + strMessage;
		strMessage += strIrregularMessage + "UnkownIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::FieldCannotRoutedIrregular:
		strMessage = strIrregularTitle + "FieldCannotRoutedIrregular>" + strMessage;
		strMessage += strIrregularMessage + "FieldCannotRoutedIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::BreakIrregular:
		strMessage = strIrregularTitle + "BreakIrregular>" + strMessage;
		strMessage += strIrregularMessage + "BreakIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ContinueIrregular:
		strMessage = strIrregularTitle + "ContinueIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ContinueIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	default:
		break;
	}

	s_pfMessageFunction(strMessage);
}

void IrisFatalErrorHandler::ShowFatalErrorDirectly(const string& strFatalErrorMessage)
{
	s_pfMessageFunction(strFatalErrorMessage);
}

IrisFatalErrorHandler::IrisFatalErrorHandler()
{
}

IrisFatalErrorHandler::~IrisFatalErrorHandler()
{
}
